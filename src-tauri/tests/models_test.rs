use hwinfo_log_viewer_lib::models::GroupsConfig;

#[test]
fn test_load_groups_config() {
    // Load the configuration file
    let config = GroupsConfig::load_from_file("config/groups.toml").unwrap();

    // print config
    println!("{:#?}", config);
}
