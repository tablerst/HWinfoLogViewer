use crate::data_processor::DataProcessor;
use crate::models::{DataGroup, GroupsConfig};
use backtrace::Backtrace;
use lazy_static::lazy_static;
use log::log;
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
            start_csv_watcher(path.clone());
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

fn start_csv_watcher(csv_path: String) {
    std::thread::spawn(move || {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher =
            RecommendedWatcher::new(tx, notify::Config::default()).expect("初始化CSV监听失败");
        watcher
            .watch(csv_path.as_ref(), RecursiveMode::NonRecursive)
            .expect("监听CSV文件失败");

        for res in rx {
            match res {
                Ok(event) if event.kind.is_modify() => {
                    for path in &event.paths {
                        if let Some(p) = path.to_str() {
                            log::info!("检测到 CSV 更新: {:?}", p);
                            // 重新处理 CSV 并更新缓存
                            let config = GLOBAL_CONFIG.read().unwrap().clone();
                            if let Ok(result) = DataProcessor::new(config).process_csv_file(p) {
                                *GLOBAL_CACHE.write().unwrap() = result;
                                log::info!("CSV 缓存已更新");
                            } else {
                                log::error!("CSV 重新处理失败: {:?}", p);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("CSV 监听错误: {:?}", e);
                }
                _ => {}
            }
        }
    });
}

fn start_config_watcher() {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher =
        RecommendedWatcher::new(tx, notify::Config::default()).expect("初始化文件监听失败");
    watcher
        .watch("config/groups.toml".as_ref(), RecursiveMode::NonRecursive)
        .expect("监听文件失败");

    std::thread::spawn(move || {
        for res in rx {
            match res {
                Ok(event) if event.kind.is_modify() => {
                    for path in &event.paths {
                        if path.ends_with("groups.toml") && event.kind.is_modify() {
                            // Load the new configuration
                            let new_cfg = GroupsConfig::load_from_file("config/groups.toml")
                                .expect("重新加载配置失败");
                            *GLOBAL_CONFIG.write().unwrap() = new_cfg;
                        }
                    }
                }
                Err(err) => {
                    eprintln!("文件监听错误: {:?}", err);
                }
                _ => {}
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::panic::set_hook(Box::new(|info| {
        let bt = Backtrace::new();
        log::error!("Panic occurred: {:?}\nBacktrace:\n{:?}", info, bt);
    }));

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Folder {
                        path: std::path::PathBuf::from("logs"),
                        file_name: Some("app.log".into()),
                    },
                ))
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|_| {
            start_config_watcher();
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
