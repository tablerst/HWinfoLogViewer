use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use csv::{ReaderBuilder, StringRecord};

use super::models::{DataGroup, FieldGroup, GroupsConfig};
use encoding_rs;
use encoding_rs_io::DecodeReaderBytesBuilder;
use regex::Regex;

pub struct DataProcessor {
    config: GroupsConfig,
    static_field_map: HashMap<String, Vec<String>>,
    regex_rules: Vec<(Regex, Vec<String>)>,
}
impl DataProcessor {
    pub fn new(config: GroupsConfig) -> Self {
        let mut static_field_map = HashMap::new();
        let mut regex_rules = Vec::new();

        for (group_name, group) in &config.groups {
            let mut path = vec![group_name.clone()];
            Self::collect_field_mappings(group, &mut path, &mut static_field_map, &mut regex_rules);
        }

        DataProcessor {
            config,
            static_field_map,
            regex_rules,
        }
    }

    fn collect_field_mappings(
        group: &FieldGroup,
        path: &mut Vec<String>,
        static_map: &mut HashMap<String, Vec<String>>,
        regex_rules: &mut Vec<(Regex, Vec<String>)>,
    ) {
        for field in &group.fields {
            static_map.insert(field.clone(), path.clone());
        }

        if let Some(pattern) = &group.field_pattern {
            if let Ok(re) = Regex::new(pattern) {
                regex_rules.push((re, path.clone()));
            }
        }

        // handle subgroups
        for (sub_name, sub_group) in &group.children {
            path.push(sub_name.clone());
            Self::collect_field_mappings(sub_group, path, static_map, regex_rules);
            path.pop();
        }
    }

    /// Read hwinfo CSV file and process it

    pub fn process_csv_file(
        &self,
        file_path: &str,
    ) -> Result<Vec<HashMap<String, DataGroup>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let buf_reader = BufReader::new(file);
        // Use encoding_rs_io to create a decoder that supports utf-8-sig (handles BOM automatically)
        let transcoded_reader = DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding_rs::UTF_8))
            .build(buf_reader);

        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(transcoded_reader);

        let headers = rdr.headers()?.clone();
        // println!("Headers Length: {:?}", headers.len());
        // println!("Headers: {:?}", headers);
        let mut field_mappings = self.build_field_mappings(&headers);

        let mut all_records = Vec::new();

        for result in rdr.records() {
            let record = result?;
            // Handle the record here, fix the last column when necessary
            let processed_record = self.process_record(&record);

            let mut data_groups = HashMap::new();

            for (header, value) in headers.iter().zip(processed_record.iter()) {
                if let Some(path) = field_mappings.get(header) {
                    self.insert_field(&mut data_groups, path, header, value.to_string());
                }
            }

            all_records.push(data_groups);
            // println!("Processed record: {:?}", processed_record);
        }

        Ok(all_records)
    }

    /// Process a single CSV record, fixing the last column if necessary
    pub fn process_record(&self, record: &StringRecord) -> Vec<String> {
        let mut processed: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        if let Some(last) = processed.last_mut() {
            *last = Self::fix_last_column(last);
        }
        processed
    }

    /// Specialized function to fix the last column of a CSV record
    ///
    /// for example:
    /// - "  test  " -> "test"
    /// - "\u{FEFF}test" -> "test"
    /// - "data\"" -> "data"
    pub fn fix_last_column(field: &str) -> String {
        let mut corrected = field.trim().to_string();
        // Remove UTF-8 BOM character (if present)
        if corrected.starts_with('\u{FEFF}') {
            corrected = corrected.trim_start_matches('\u{FEFF}').to_string();
        }
        // When last column ends with a double quote, but does not start with one, remove the last double quote
        if corrected.ends_with('"') && !corrected.starts_with('"') {
            corrected.pop();
        }
        corrected
    }

    /// insert a field into the corresponding data group structure
    fn insert_field(
        &self,
        data_groups: &mut HashMap<String, DataGroup>,
        path: &[String],
        field_name: &str,
        value: String,
    ) {
        if let Some(group_name) = path.first() {
            let group = data_groups
                .entry(group_name.clone())
                .or_insert_with(|| DataGroup::new());
            group.insert(&path[1..], field_name, value);
        }
    }

    fn build_field_mappings(&self, headers: &StringRecord) -> HashMap<String, Vec<String>> {
        let mut mappings = self.static_field_map.clone();

        for header in headers.iter() {
            if !mappings.contains_key(header) {
                for (re, path) in &self.regex_rules {
                    if re.is_match(header) {
                        mappings.insert(header.to_string(), path.clone());
                        break;
                    }
                }
            }
        }

        mappings
    }
}
