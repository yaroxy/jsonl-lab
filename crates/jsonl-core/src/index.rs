use anyhow::{Context, Result, bail};
use memchr::memchr_iter;
use memmap2::Mmap;
use serde::Serialize;
use std::fs::{File, Metadata};
use std::io::{Read, Write};
use std::path::Path;
use std::time::UNIX_EPOCH;

const INDEX_MAGIC: &[u8; 4] = b"JIDX";
const INDEX_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize)]
pub struct JsonlIndex {
    pub meta: IndexMeta,
    pub offsets: Vec<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexMeta {
    pub data_file_size: u64,
    pub data_modified_secs: u64,
    pub data_modified_nanos: u32,
    pub num_lines: u64,
}

pub fn build_index(path: impl AsRef<Path>) -> Result<JsonlIndex> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let mmap = unsafe { Mmap::map(&file)? };

    let mut offsets = Vec::new();

    if !mmap.is_empty() {
        offsets.push(0);
    }

    for pos in memchr_iter(b'\n', &mmap) {
        let next = pos + 1;
        if next < mmap.len() {
            offsets.push(next as u64);
        }
    }

    let meta = index_meta_from_metadata(&metadata, offsets.len())?;

    Ok(JsonlIndex { meta, offsets })
}

pub fn save_index(path: impl AsRef<Path>, index: &JsonlIndex) -> Result<()> {
    validate_index_offsets(index)?;

    let mut file = File::create(path)?;

    file.write_all(INDEX_MAGIC)?;
    file.write_all(&INDEX_VERSION.to_le_bytes())?;
    file.write_all(&index.meta.data_file_size.to_le_bytes())?;
    file.write_all(&index.meta.data_modified_secs.to_le_bytes())?;
    file.write_all(&index.meta.data_modified_nanos.to_le_bytes())?;
    file.write_all(&index.meta.num_lines.to_le_bytes())?;

    for &offset in &index.offsets {
        file.write_all(&offset.to_le_bytes())?;
    }

    Ok(())
}

pub fn load_index(path: impl AsRef<Path>) -> Result<JsonlIndex> {
    let mut file = File::open(path)?;

    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    anyhow::ensure!(&magic == INDEX_MAGIC, "invalid index file magic");

    let mut version = [0u8; 4];
    file.read_exact(&mut version)?;
    let version = u32::from_le_bytes(version);
    anyhow::ensure!(
        version == INDEX_VERSION,
        "unsupported index version {}; rerun `jsonl-lab index`",
        version
    );

    let mut file_size_buf = [0u8; 8];
    file.read_exact(&mut file_size_buf)?;
    let data_file_size = u64::from_le_bytes(file_size_buf);

    let mut modified_secs_buf = [0u8; 8];
    file.read_exact(&mut modified_secs_buf)?;
    let data_modified_secs = u64::from_le_bytes(modified_secs_buf);

    let mut modified_nanos_buf = [0u8; 4];
    file.read_exact(&mut modified_nanos_buf)?;
    let data_modified_nanos = u32::from_le_bytes(modified_nanos_buf);

    let mut len_buf = [0u8; 8];
    file.read_exact(&mut len_buf)?;
    let num_lines = u64::from_le_bytes(len_buf);
    let len = usize::try_from(num_lines).context("index is too large for this platform")?;

    let mut offsets = Vec::with_capacity(len);

    for _ in 0..len {
        let mut buf = [0u8; 8];
        file.read_exact(&mut buf)?;
        offsets.push(u64::from_le_bytes(buf));
    }

    let index = JsonlIndex {
        meta: IndexMeta {
            data_file_size,
            data_modified_secs,
            data_modified_nanos,
            num_lines,
        },
        offsets,
    };

    validate_index_offsets(&index)?;

    Ok(index)
}

pub fn validate_index(data_path: impl AsRef<Path>, index: &JsonlIndex) -> Result<()> {
    let data_path = data_path.as_ref();
    let metadata = std::fs::metadata(data_path)?;
    let current = index_meta_from_metadata(&metadata, index.offsets.len())?;

    if current.data_file_size != index.meta.data_file_size
        || current.data_modified_secs != index.meta.data_modified_secs
        || current.data_modified_nanos != index.meta.data_modified_nanos
    {
        bail!(
            "index is stale for {}; run `jsonl-lab index {}`",
            data_path.display(),
            data_path.display()
        );
    }

    Ok(())
}

fn index_meta_from_metadata(metadata: &Metadata, num_lines: usize) -> Result<IndexMeta> {
    let modified = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .context("file modified time is before unix epoch")?;

    Ok(IndexMeta {
        data_file_size: metadata.len(),
        data_modified_secs: modified.as_secs(),
        data_modified_nanos: modified.subsec_nanos(),
        num_lines: num_lines as u64,
    })
}

fn validate_index_offsets(index: &JsonlIndex) -> Result<()> {
    let expected_len =
        usize::try_from(index.meta.num_lines).context("index is too large for this platform")?;

    anyhow::ensure!(
        expected_len == index.offsets.len(),
        "index line count does not match offset count"
    );

    if index.offsets.is_empty() {
        return Ok(());
    }

    anyhow::ensure!(index.offsets[0] == 0, "first index offset must be 0");

    for window in index.offsets.windows(2) {
        anyhow::ensure!(window[0] < window[1], "index offsets must be sorted");
    }

    for &offset in &index.offsets {
        anyhow::ensure!(
            offset <= index.meta.data_file_size,
            "index offset exceeds data file size"
        );
    }

    Ok(())
}
