use crate::data_processor::DataProcessor;
use crate::models::{DataGroup, GroupsConfig};
use lazy_static::lazy_static;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::mpsc::channel;
use std::sync::RwLock;

pub mod data_processor;
pub mod models;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
fn load_csv(path: String) -> Result<(), String> {
    println!("Starting CSV processing..., path: {:?}", path);
    let processor = DataProcessor::new(GLOBAL_CONFIG.read().unwrap().clone());
    match processor.process_csv_file(&path) {
        Ok(result) => {
            *GLOBAL_CACHE.write().unwrap() = result;
            Ok(())
        }
        Err(e) => Err(format!("CSV 处理失败: {:?}", e)),
    }
}

#[tauri::command]
fn get_data() -> String {
    serde_json::to_string(&GLOBAL_CACHE.read().unwrap().clone()).unwrap()
}

#[tauri::command]
fn get_data_by_key(key: String) -> String {
    let cache = GLOBAL_CACHE.read().unwrap();
    let mut result: Vec<DataGroup> = Vec::new();

    for group in cache.iter() {
        // println!("Group: {:?}, Key: {:?}", group, key);
        result.extend(find_key_in_group(group, &key));
    }

    serde_json::to_string(&result).unwrap()
}

pub fn find_key_in_group(group: &HashMap<String, DataGroup>, key: &str) -> Vec<DataGroup> {
    let mut found: Vec<DataGroup> = Vec::new();

    let mut new_group = DataGroup::new();

    if let Some(base_data) = group.get("base") {
        // If there is base data, return it directly
        new_group.fields.extend(base_data.fields.clone());
    }

    // 1. Check if the current group contains the key in its fields
    for data_group in group.values() {
        println!("DataGroup: {:?}", data_group);
        if let Some(value) = data_group.fields.get(key) {
            new_group.fields.insert(key.to_string(), value.clone());
            found.push(new_group.clone());
        }
    }

    // 2. Check if the current group contains the key in its children
    for child in group.values() {
        found.extend(find_key_in_group(&child.children, key));
    }

    found
}

lazy_static! {
    static ref GLOBAL_CONFIG: RwLock<GroupsConfig> = {
        let cfg = GroupsConfig::load_from_file("config/groups.toml").expect("加载配置失败");
        RwLock::new(cfg)
    };
    static ref GLOBAL_CACHE: RwLock<Vec<HashMap<String, DataGroup>>> = RwLock::new(Vec::new());
}
fn spawn_file_watcher() {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher =
        RecommendedWatcher::new(tx, notify::Config::default()).expect("初始化文件监听失败");
    watcher
        .watch("config/groups.toml".as_ref(), RecursiveMode::NonRecursive)
        .expect("监听文件失败");
    watcher
        .watch("data/1.CSV".as_ref(), RecursiveMode::NonRecursive)
        .expect("监听文件失败");

    std::thread::spawn(move || {
        for res in rx {
            match res {
                Ok(event) => {
                    for path in &event.paths {
                        if path.ends_with("groups.toml") && event.kind.is_modify() {
                            // Load the new configuration
                            let new_cfg = GroupsConfig::load_from_file("config/groups.toml")
                                .expect("重新加载配置失败");
                            *GLOBAL_CONFIG.write().unwrap() = new_cfg;
                        } else if path.ends_with(".CSV") && event.kind.is_modify() {
                            println!("检测到 CSV 更新: {:?}", path);

                            let config = GLOBAL_CONFIG.read().unwrap().clone();
                            let processor = DataProcessor::new(config);
                            let result: Vec<HashMap<String, DataGroup>> =
                                processor.process_csv_file("data/1.CSV").unwrap();
                            *GLOBAL_CACHE.write().unwrap() = result;
                        }
                    }
                }
                Err(err) => {
                    eprintln!("文件监听错误: {:?}", err);
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|_| {
            spawn_file_watcher();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_data,
            get_data_by_key,
            load_csv
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
