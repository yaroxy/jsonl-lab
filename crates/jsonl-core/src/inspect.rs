use crate::JsonlDataset;
use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct InspectReport {
    pub path: String,
    pub file_size: u64,
    pub num_lines: usize,
    pub start: usize,
    pub requested_sample: usize,
    pub sample_size: usize,
    pub valid_json: usize,
    pub invalid_json: usize,
    pub top_level: BTreeMap<String, usize>,
    pub fields: BTreeMap<String, FieldStats>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct FieldStats {
    pub present: usize,
    pub types: BTreeMap<String, usize>,
}

pub fn inspect_dataset(
    dataset: &JsonlDataset,
    start: usize,
    sample: usize,
) -> Result<InspectReport> {
    let end = start.saturating_add(sample).min(dataset.len());
    let sample_size = end.saturating_sub(start);

    let mut report = InspectReport {
        path: dataset.path.display().to_string(),
        file_size: dataset.file_size as u64,
        num_lines: dataset.len(),
        start,
        requested_sample: sample,
        sample_size,
        valid_json: 0,
        invalid_json: 0,
        top_level: BTreeMap::new(),
        fields: BTreeMap::new(),
    };

    for idx in start..end {
        let line = dataset.raw_line(idx)?;

        match serde_json::from_slice::<Value>(line) {
            Ok(value) => {
                report.valid_json += 1;

                let kind = value_kind(&value).to_string();
                *report.top_level.entry(kind).or_default() += 1;

                if let Value::Object(object) = value {
                    for (field, field_value) in object {
                        let stats = report.fields.entry(field).or_default();
                        stats.present += 1;

                        let kind = value_kind(&field_value).to_string();
                        *stats.types.entry(kind).or_default() += 1;
                    }
                }
            }
            Err(_) => {
                report.invalid_json += 1;
            }
        }
    }

    Ok(report)
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
