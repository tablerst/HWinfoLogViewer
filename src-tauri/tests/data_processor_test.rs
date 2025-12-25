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
fn test_process_csv_file_with_tail_group_overlay() -> Result<(), Box<dyn std::error::Error>> {
    let config = GroupsConfig::load_from_file("config/groups.toml").unwrap();
    let processor = DataProcessor::new(config);

    let result = processor.process_csv_file("data/min_tail_group.CSV")?;
    assert!(!result.is_empty());

    let first = &result[0];

    // Date/Time must stay in base
    assert!(first.contains_key("base"));

    // CPU group should be overridden by CSV parent title
    assert!(first.contains_key("CPU [#0]: AMD Ryzen 9 5950X"));

    // Drive group should be overridden as well; note the unquoted comma inside brackets
    assert!(first.contains_key("Drive: Lexar SSD NM620 2TB [E:, F:]"));

    Ok(())
}

#[test]
fn test_process_csv_file_invalid_mid_header_rejected() {
    let config = GroupsConfig::load_from_file("config/groups.toml").unwrap();
    let processor = DataProcessor::new(config);

    let err = processor
        .process_csv_file("data/invalid_mid_header.CSV")
        .expect_err("should reject CSV files that repeat the subtitle header in the middle");

    let msg = err.to_string();
    assert!(
        msg.contains("重复表头") || msg.contains("子标题表头"),
        "unexpected error message: {msg}"
    );
}

#[test]
fn test_process_csv_file_mid_log_extra_column_tolerated() -> Result<(), Box<dyn std::error::Error>> {
    let config = GroupsConfig::load_from_file("config/groups.toml").unwrap();
    let processor = DataProcessor::new(config);

    // This fixture simulates HWiNFO adding a sensor mid-log without rewriting the header,
    // resulting in one extra column from that point onward.
    let result = processor.process_csv_file("data/mid_log_extra_column.CSV")?;
    assert_eq!(result.len(), 3);

    for row in &result {
        let base = row.get("base").expect("base group should exist");
        assert!(base.fields.contains_key("Date"));
        assert!(base.fields.contains_key("Time"));
    }

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
