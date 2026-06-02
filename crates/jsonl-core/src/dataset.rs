use anyhow::{Result, bail};
use memmap2::Mmap;
use serde_json::Value;
use std::fs::File;
use std::path::{Path, PathBuf};

pub struct JsonlDataset {
    pub path: PathBuf,
    pub file_size: usize,
    pub offsets: Vec<u64>,
    mmap: Mmap,
}

impl JsonlDataset {
    pub fn open(path: impl AsRef<Path>, offsets: Vec<u64>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let file_size = mmap.len();

        Ok(Self {
            path,
            file_size,
            offsets,
            mmap,
        })
    }

    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    pub fn raw_line(&self, idx: usize) -> Result<&[u8]> {
        if idx >= self.offsets.len() {
            bail!(
                "index out of range: idx={}, len={}",
                idx,
                self.offsets.len()
            );
        }

        let start = self.offsets[idx] as usize;
        let end = if idx + 1 < self.offsets.len() {
            self.offsets[idx + 1] as usize
        } else {
            self.file_size
        };

        if start > end || end > self.file_size {
            bail!(
                "index offset range is invalid: start={}, end={}",
                start,
                end
            );
        }

        let mut line = &self.mmap[start..end];

        if line.ends_with(b"\n") {
            line = &line[..line.len() - 1];
        }
        if line.ends_with(b"\r") {
            line = &line[..line.len() - 1];
        }

        Ok(line)
    }

    pub fn json_value(&self, idx: usize) -> Result<Value> {
        let line = self.raw_line(idx)?;
        let value = serde_json::from_slice(line)?;
        Ok(value)
    }

    pub fn range_raw(&self, start: usize, limit: usize) -> Result<Vec<String>> {
        let end = start.saturating_add(limit).min(self.len());
        let mut out = Vec::with_capacity(end.saturating_sub(start));

        for idx in start..end {
            let line = self.raw_line(idx)?;
            out.push(String::from_utf8_lossy(line).to_string());
        }

        Ok(out)
    }
}
