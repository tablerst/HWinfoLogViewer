use encoding_rs;

use hwinfo_log_viewer_lib::data_processor::DataProcessor;
use hwinfo_log_viewer_lib::find_key_in_group;
use hwinfo_log_viewer_lib::models::GroupsConfig;
use serde_json;

/// Test the fix_last_column function to ensure it correctly removes BOM, trims whitespace, and handles extra quotes
#[test]
fn test_fix_last_column() {
    // Test removing BOM
    let input = "\u{FEFF}test";
    let expected = "test".to_string();
    assert_eq!(DataProcessor::fix_last_column(input), expected);

    // Test trimming whitespace
    let input = "  test  ";
    let expected = "test".to_string();
    assert_eq!(DataProcessor::fix_last_column(input), expected);

    // Test removing extra quotes
    let input = "data\"";
    let expected = "data".to_string();
    assert_eq!(DataProcessor::fix_last_column(input), expected);

    // Test that valid quotes are not modified (e.g., field is surrounded by quotes)
    let input = "\"data\"";
    let expected = "\"data\"".to_string();
    assert_eq!(DataProcessor::fix_last_column(input), expected);
}

#[test]
fn test_process_csv_file() -> Result<(), Box<dyn std::error::Error>> {
    let config = GroupsConfig::load_from_file("config/groups.toml").unwrap();

    println!("Config: {:?}", config);
    // Test the CSV file processing
    let processor = DataProcessor::new(config);
    let result = processor.process_csv_file("data/1.CSV")?;
    // 访问示例数据
    println!("Result: {:?}", result);

    Ok(())
}

#[test]
fn test_find_key_in_group() {
    // Test the find_key_in_group function
    let config = GroupsConfig::load_from_file("config/groups.toml").unwrap();

    let processor = DataProcessor::new(config);
    let result = processor.process_csv_file("data/1.CSV");

    let mut res = Vec::new();
    let key = "GPU";

    for group in result.unwrap().iter() {
        println!("Group: {:?}, Key: {:?}", group, key);
        res.extend(find_key_in_group(group, &key));
    }

    let data = serde_json::to_string(&res).unwrap();

    println!("Data: {}", data);
}
