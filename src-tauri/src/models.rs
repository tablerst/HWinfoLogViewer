use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;

/// Supplementary structure for deserializing TOML configuration
#[derive(Debug, Deserialize)]
struct RawGroup {
    fields: Option<Vec<String>>,
    field_pattern: Option<String>,
    #[serde(flatten)]
    subgroups: Option<HashMap<String, RawGroup>>,
}

/// DynamicConfig to represent a group, including group name, fields, and possible subgroups
#[derive(Debug, Clone)]
pub struct FieldGroup {
    pub fields: Vec<String>,
    pub field_pattern: Option<String>,
    pub children: HashMap<String, FieldGroup>,
}

impl FieldGroup {
    pub fn new(fields: Vec<String>, field_pattern: Option<String>) -> Self {
        FieldGroup {
            fields,
            field_pattern,
            children: HashMap::new(),
        }
    }

    /// From RawGroup to FieldGroup
    pub fn from_raw(raw: &RawGroup) -> Self {
        let fields = raw.fields.clone().unwrap_or_default();
        let field_pattern = raw.field_pattern.clone();
        let mut group = FieldGroup::new(fields, field_pattern);
        if let Some(subs) = &raw.subgroups {
            for (subname, raw_sub) in subs {
                group
                    .children
                    .insert(subname.clone(), FieldGroup::from_raw(raw_sub));
            }
        }
        group
    }
}

/// GroupsConfig save all top groups
#[derive(Debug, Clone)]
pub struct GroupsConfig {
    pub groups: HashMap<String, FieldGroup>,
}

impl std::fmt::Display for GroupsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GroupsConfig {{")?;
        for (name, group) in &self.groups {
            writeln!(
                f,
                "  {}: {} fields, {} children",
                name,
                group.fields.len(),
                group.children.len()
            )?;
        }
        write!(f, "}}")
    }
}

impl GroupsConfig {
    /// Load configuration from a TOML file
    pub fn load_from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(file_path)?;
        // Directly deserialize the TOML top level into HashMap<String, RawGroup>
        let raw_map: HashMap<String, RawGroup> = toml::from_str(&content)?;
        let groups = raw_map
            .into_iter()
            .map(|(name, raw)| (name, FieldGroup::from_raw(&raw)))
            .collect();
        Ok(GroupsConfig { groups })
    }
}

/// save processed data
#[derive(Debug, serde::Serialize, Clone)]
pub struct DataGroup {
    #[serde(flatten)]
    pub fields: HashMap<String, String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub children: HashMap<String, DataGroup>,
}

impl DataGroup {
    pub fn new() -> Self {
        DataGroup {
            fields: HashMap::new(),
            children: HashMap::new(),
        }
    }

    /// recursive insert field to the corresponding subgroup
    pub fn insert(&mut self, remaining_path: &[String], field_name: &str, value: String) {
        if remaining_path.is_empty() {
            self.fields.insert(field_name.to_string(), value);
        } else {
            let child_name = &remaining_path[0];
            let child = self
                .children
                .entry(child_name.clone())
                .or_insert_with(|| DataGroup::new());
            child.insert(&remaining_path[1..], field_name, value);
        }
    }
}

#[derive(serde::Serialize)]
struct JsonDataGroup {
    fields: HashMap<String, String>,
    children: Vec<JsonDataGroup>,
}

impl From<&DataGroup> for JsonDataGroup {
    fn from(d: &DataGroup) -> Self {
        JsonDataGroup {
            fields: d.fields.clone(),
            children: d.children.values().map(|v| v.into()).collect(),
        }
    }
}
