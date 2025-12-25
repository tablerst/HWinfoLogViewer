use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

use csv::{ReaderBuilder, StringRecord};

use super::models::{DataGroup, FieldGroup, GroupsConfig};
use encoding_rs;
use encoding_rs_io::DecodeReaderBytesBuilder;
use regex::Regex;

#[derive(Default, Debug)]
struct RecordAlignmentState {
    /// When HWiNFO starts writing an extra column mid-log (without rewriting the header),
    /// we infer the insertion position once and drop that field for subsequent records.
    drop_index_for_extra_col: Option<usize>,
    prev_record: Option<Vec<String>>,
}

pub struct DataProcessor {
    config: GroupsConfig,
    static_field_map: HashMap<String, Vec<String>>,
    regex_rules: Vec<(Regex, Vec<String>)>,
}

#[derive(Debug, Clone)]
struct TailGroupMeta {
    /// Index in `lines` of the repeated header line (subtitle header).
    repeated_header_idx: usize,
    /// Index in `lines` of the parent-title line.
    parent_line_idx: usize,
    /// Raw text of the parent-title line.
    parent_line: String,
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
        // 1) Read file as UTF-8 text lines (BOM handled by decoder), then strictly validate tail meta format.
        let file = File::open(file_path)?;
        let buf_reader = BufReader::new(file);
        let transcoded_reader = DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding_rs::UTF_8))
            .build(buf_reader);

        let mut line_reader = BufReader::new(transcoded_reader);
        let mut lines: Vec<String> = Vec::new();
        let mut buf = String::new();
        while line_reader.read_line(&mut buf)? != 0 {
            if buf.ends_with('\n') {
                buf.pop();
                if buf.ends_with('\r') {
                    buf.pop();
                }
            }
            lines.push(std::mem::take(&mut buf));
        }

        let (header_line, tail_meta) = Self::analyze_hwinfo_lines(&lines)?;

        // 2) Reconstruct the CSV content excluding tail meta lines (if present) and parse data strictly.
        let mut csv_content = String::new();
        for (idx, line) in lines.iter().enumerate() {
            if let Some(meta) = &tail_meta {
                if idx == meta.repeated_header_idx || idx == meta.parent_line_idx {
                    continue;
                }
            }
            csv_content.push_str(line);
            csv_content.push('\n');
        }

        let cursor = Cursor::new(csv_content);
        // HWiNFO logs are mostly regular CSV, but can occasionally change sensor set mid-file
        // without rewriting the header. We enable `flexible` and normalize record lengths ourselves.
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(cursor);

        let raw_headers = rdr.headers()?.clone();
        let headers = Self::normalize_headers(&raw_headers);

        // 3) Build group mappings: base from groups.toml + optional overlay from CSV tail meta.
        let overlay = match &tail_meta {
            Some(meta) => match Self::build_overlay_from_parent_line(&headers, &meta.parent_line) {
                Ok(map) => Some(map),
                Err(e) => {
                    // If the CSV claims to provide tail grouping but it cannot be parsed reliably,
                    // degrade gracefully to groups.toml.
                    log::warn!(
                        "检测到尾部分组元数据但解析失败，将回退到 groups.toml；path={:?}, err={:?}",
                        file_path,
                        e
                    );
                    None
                }
            },
            None => None,
        };

        let field_mappings = self.build_field_mappings_with_overlay(&headers, overlay.as_ref());

        let mut all_records = Vec::new();
        let mut align_state = RecordAlignmentState::default();
        for result in rdr.records() {
            let record = result?;
            let processed_record = Self::normalize_and_process_record(&record, headers.len(), &mut align_state)?;

            let mut data_groups = HashMap::new();
            for (header, value) in headers.iter().zip(processed_record.iter()) {
                if let Some(path) = field_mappings.get(header) {
                    self.insert_field(&mut data_groups, path, header, value.to_string());
                }
            }

            all_records.push(data_groups);
        }

        // Sanity: ensure the detected header line really was the first subtitle header.
        // This is just to silence unused variable warnings in case future refactors remove it.
        let _ = header_line;

        Ok(all_records)
    }

    /// Normalize headers by trimming trailing empty columns (commonly produced by a trailing comma).
    fn normalize_headers(raw_headers: &StringRecord) -> Vec<String> {
        let mut headers: Vec<String> = raw_headers.iter().map(|s| s.to_string()).collect();
        while headers.last().map(|h| h.trim().is_empty()).unwrap_or(false) {
            headers.pop();
        }
        headers
    }

    /// Strictly validate HWiNFO CSV structure.
    ///
    /// We only accept exactly two formats:
    /// 1) subtitle header appears only once at the top
    /// 2) subtitle header appears at the top + repeated at the penultimate non-empty line,
    ///    and the last non-empty line is the parent-title row (Date/Time empty).
    fn analyze_hwinfo_lines(lines: &[String]) -> Result<(String, Option<TailGroupMeta>), Box<dyn Error>> {
        let non_empty_indices: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                let n = Self::normalize_for_compare(s);
                if n.is_empty() { None } else { Some(i) }
            })
            .collect();

        if non_empty_indices.is_empty() {
            return Err("CSV 文件为空".into());
        }

        let header_idx = non_empty_indices[0];
        let header_line = lines[header_idx].clone();

        // Count occurrences of any line that looks like the subtitle header (starts with Date,Time).
        let header_like_indices: Vec<usize> = non_empty_indices
            .iter()
            .copied()
            .filter(|&i| Self::is_subtitle_header_line(&lines[i]))
            .collect();

        if header_like_indices.is_empty() || header_like_indices[0] != header_idx {
            return Err("CSV 文件不符合预期：首个非空行不是子标题表头（Date,Time,...）".into());
        }

        let last_idx = *non_empty_indices.last().unwrap();
        let last_line_norm = Self::normalize_for_compare(&lines[last_idx]);

        match header_like_indices.len() {
            1 => {
                // Strictly reject the "parent-title-only" tail shape.
                if last_line_norm.starts_with(",,") {
                    return Err(
                        "检测到疑似父标题行（Date/Time 为空）但未检测到倒数第二行的重复表头；仅支持：只在头部出现表头，或头部+尾部两行（重复表头+父标题行）"
                            .into(),
                    );
                }
                Ok((header_line, None))
            }
            2 => {
                if non_empty_indices.len() < 2 {
                    return Err("CSV 文件内容不足，无法判定尾部分组元数据".into());
                }
                let penultimate_idx = non_empty_indices[non_empty_indices.len() - 2];
                let second_header_idx = header_like_indices[1];
                if second_header_idx != penultimate_idx {
                    return Err(
                        "检测到多次子标题表头，但重复表头不在倒数第二个非空行；仅支持：表头只在头部出现，或头部+尾部两行（重复表头+父标题行）"
                            .into(),
                    );
                }
                if !last_line_norm.starts_with(",,") {
                    return Err(
                        "检测到倒数第二行是重复表头，但最后一行不是父标题行（Date/Time 为空）"
                            .into(),
                    );
                }

                Ok((
                    header_line,
                    Some(TailGroupMeta {
                        repeated_header_idx: second_header_idx,
                        parent_line_idx: last_idx,
                        parent_line: lines[last_idx].clone(),
                    }),
                ))
            }
            _ => Err(
                "检测到多次子标题表头（Date,Time,...）出现次数超过 2；仅支持：表头只在头部出现，或头部+尾部两行（重复表头+父标题行）"
                    .into(),
            ),
        }
    }

    fn normalize_for_compare(s: &str) -> String {
        let mut out = s.trim().to_string();
        if out.starts_with('\u{FEFF}') {
            out = out.trim_start_matches('\u{FEFF}').to_string();
        }
        out
    }

    fn is_subtitle_header_line(line: &str) -> bool {
        let n = Self::normalize_for_compare(line);
        // Be intentionally strict: HWiNFO exports always start with Date,Time
        n.starts_with("Date,Time")
    }

    /// Build an overlay map: header field -> CSV parent group name (top-level group).
    ///
    /// This parses the parent-title line (last line) using a tolerant splitter that
    /// treats commas inside square brackets (e.g. "[C:, D:]") as normal characters.
    fn build_overlay_from_parent_line(
        headers: &[String],
        parent_line: &str,
    ) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let mut fields = Self::split_parent_title_line(parent_line);

        // Strip trailing empty fields commonly produced by a trailing comma.
        while fields.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
            fields.pop();
        }

        if fields.len() < headers.len() {
            fields.resize(headers.len(), String::new());
        }

        if fields.len() != headers.len() {
            return Err(format!(
                "父标题行列数无法对齐：headers_len={}, parent_len={}",
                headers.len(),
                fields.len()
            )
            .into());
        }

        let mut overlay = HashMap::new();
        for (idx, header) in headers.iter().enumerate() {
            if header == "Date" || header == "Time" {
                continue;
            }
            let parent = fields[idx].trim();
            if parent.is_empty() {
                continue;
            }
            overlay.insert(header.to_string(), parent.to_string());
        }
        Ok(overlay)
    }

    /// Split the parent-title line by commas, but do NOT split commas inside quotes
    /// or inside square brackets `[...]`.
    fn split_parent_title_line(line: &str) -> Vec<String> {
        let mut fields: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut bracket_depth: u32 = 0;

        for ch in line.chars() {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                    current.push(ch);
                }
                '[' if !in_quotes => {
                    bracket_depth = bracket_depth.saturating_add(1);
                    current.push(ch);
                }
                ']' if !in_quotes => {
                    bracket_depth = bracket_depth.saturating_sub(1);
                    current.push(ch);
                }
                ',' if !in_quotes && bracket_depth == 0 => {
                    fields.push(std::mem::take(&mut current));
                }
                _ => current.push(ch),
            }
        }
        fields.push(current);
        fields
    }

    /// Normalize record length to match `expected_len` and apply minimal field cleanup.
    ///
    /// This handles two common HWiNFO quirks:
    /// - A trailing comma produces an extra empty field (we trim trailing empty fields).
    /// - The sensor set can change mid-log, adding one extra column without rewriting headers.
    ///   In that case we infer the insertion point once (via adjacent-record alignment) and
    ///   drop the extra field for subsequent records.
    fn normalize_and_process_record(
        record: &StringRecord,
        expected_len: usize,
        state: &mut RecordAlignmentState,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();

        // Trim trailing empty fields (common if the exporter always ends lines with a comma).
        while fields.len() > expected_len
            && fields.last().map(|s| s.trim().is_empty()).unwrap_or(false)
        {
            fields.pop();
        }

        if fields.len() == expected_len {
            // ok
        } else if fields.len() == expected_len + 1 {
            let drop_idx = match state.drop_index_for_extra_col {
                Some(idx) => idx,
                None => {
                    // Infer drop index using previous normalized record (if available).
                    let idx = if let Some(prev) = state.prev_record.as_ref() {
                        Self::infer_drop_index_for_extra_column(prev, &fields)
                    } else {
                        // No context to infer; drop the extra field at the end as a conservative fallback.
                        expected_len
                    };
                    state.drop_index_for_extra_col = Some(idx);
                    idx
                }
            };

            if drop_idx >= fields.len() {
                return Err(format!(
                    "记录列数异常：预期 {}，实际 {}，推断丢弃位置 {} 越界",
                    expected_len,
                    fields.len(),
                    drop_idx
                )
                .into());
            }
            fields.remove(drop_idx);
        } else if fields.len() < expected_len {
            // Be lenient for missing tail columns: pad with empty strings.
            fields.resize(expected_len, String::new());
        } else {
            return Err(format!(
                "CSV 记录列数不一致：预期 {}，实际 {}；目前仅支持相差 1 列的传感器动态增量情况",
                expected_len,
                fields.len()
            )
            .into());
        }

        // Minimal cleanup for the last column.
        if let Some(last) = fields.last_mut() {
            *last = Self::fix_last_column(last);
        }

        // Update prev_record for alignment inference.
        state.prev_record = Some(fields.clone());

        Ok(fields)
    }

    /// Infer which field to drop from `curr` (len = prev.len()+1) to best align with `prev`.
    fn infer_drop_index_for_extra_column(prev: &[String], curr: &[String]) -> usize {
        debug_assert_eq!(curr.len(), prev.len() + 1);

        // Skip Date/Time for scoring because they always change and are not numeric.
        let start_idx = 2usize.min(prev.len());

        let mut best_idx: usize = prev.len();
        let mut best_score: f64 = f64::INFINITY;

        for k in 0..curr.len() {
            let mut score = 0.0f64;
            let mut used = 0u32;

            for i in start_idx..prev.len() {
                let a = prev[i].parse::<f64>();
                let b_str = if i < k { &curr[i] } else { &curr[i + 1] };
                let b = b_str.parse::<f64>();
                if let (Ok(av), Ok(bv)) = (a, b) {
                    score += (av - bv).abs();
                    used += 1;
                }
                if score >= best_score {
                    break;
                }
            }

            // If we couldn't compare any numeric fields (unlikely), fall back to dropping near the end.
            if used == 0 {
                score = (prev.len() as f64 - k as f64).abs();
            }

            if score < best_score {
                best_score = score;
                best_idx = k;
            }
        }

        best_idx
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

    fn build_field_mappings(&self, headers: &[String]) -> HashMap<String, Vec<String>> {
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

    /// Build field mappings from groups.toml, then overlay top-level group name from CSV (if provided).
    ///
    /// Overlay semantics:
    /// - If CSV provides a parent group for a column, it overrides ONLY the first segment of the path.
    /// - If the field is unknown to groups.toml but CSV provides a parent group, we create a 1-level path.
    /// - Date/Time always remain in `base`.
    fn build_field_mappings_with_overlay(
        &self,
        headers: &[String],
        overlay_top_group: Option<&HashMap<String, String>>,
    ) -> HashMap<String, Vec<String>> {
        let mut mappings = self.build_field_mappings(headers);

        let Some(overlay) = overlay_top_group else {
            return mappings;
        };

        for header in headers.iter() {
            if header == "Date" || header == "Time" {
                continue;
            }
            let Some(parent) = overlay.get(header) else {
                continue;
            };
            let parent = parent.trim();
            if parent.is_empty() {
                continue;
            }

            match mappings.get_mut(header) {
                Some(path) if !path.is_empty() => {
                    // Do not override base.
                    if path[0] != "base" {
                        path[0] = parent.to_string();
                    }
                }
                Some(path) => {
                    *path = vec![parent.to_string()];
                }
                None => {
                    mappings.insert(header.to_string(), vec![parent.to_string()]);
                }
            }
        }

        mappings
    }
}
