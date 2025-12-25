use crate::data_processor::DataProcessor;
use crate::models::{DataGroup, GroupsConfig};
use backtrace::Backtrace;
use lazy_static::lazy_static;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::sync::RwLock;

pub mod data_processor;
pub mod models;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
fn load_csv(path: String) -> Result<(), String> {
    log::info!("Starting CSV processing..., path: {:?}", path);
    let config = GLOBAL_CONFIG
        .read()
        .map_err(|e| format!("读取配置失败（锁已污染）: {e}"))?
        .clone();
    let processor = DataProcessor::new(config);
    match processor.process_csv_file(&path) {
        Ok(result) => {
            *GLOBAL_CACHE
                .write()
                .map_err(|e| format!("更新缓存失败（锁已污染）: {e}"))? = result;
            start_csv_watcher(path.clone());
            log::info!("CSV processed successfully: {:?}", path);
            Ok(())
        }
        Err(e) => {
            log::error!("CSV 处理失败, path={:?}, err={:?}", path, e);
            Err(format!("CSV 处理失败（{path}）: {e}"))
        }
    }
}

#[tauri::command]
fn get_data() -> Result<String, String> {
    let cache = GLOBAL_CACHE
        .read()
        .map_err(|e| format!("读取缓存失败（锁已污染）: {e}"))?
        .clone();
    serde_json::to_string(&cache).map_err(|e| {
        log::error!("序列化缓存失败: {e}");
        format!("序列化缓存失败: {e}")
    })
}

#[tauri::command]
fn get_data_by_key(key: String) -> Result<String, String> {
    if key.trim().is_empty() {
        return Err("key 不能为空".to_string());
    }

    let cache = GLOBAL_CACHE
        .read()
        .map_err(|e| format!("读取缓存失败（锁已污染）: {e}"))?;
    let mut result: Vec<DataGroup> = Vec::new();

    for group in cache.iter() {
        // println!("Group: {:?}, Key: {:?}", group, key);
        result.extend(find_key_in_group(group, &key));
    }

    serde_json::to_string(&result).map_err(|e| {
        log::error!("序列化 key={:?} 的结果失败: {e}", key);
        format!("序列化结果失败: {e}")
    })
}

pub fn find_key_in_group(group: &HashMap<String, DataGroup>, key: &str) -> Vec<DataGroup> {
    let mut found = Vec::new();
    
    // Handle top-level data
    if let Some(base_data) = group.get("base") {
        found.extend(search_in_group(group, key, Some(base_data)));
    } else {
        found.extend(search_in_group(group, key, None));
    }

    found
}

fn search_in_group(
    group: &HashMap<String, DataGroup>,
    key: &str,
    base_data: Option<&DataGroup>
) -> Vec<DataGroup> {
    let mut found = Vec::new();
    let mut new_group = DataGroup::new();
    
    // slove base_data
    if let Some(base_data) = base_data {
        new_group.fields.extend(base_data.fields.clone());
    }
    
    // 1. search if current group contains key
    for data_group in group.values() {
        if let Some(value) = data_group.fields.get(key) {
            new_group.fields.insert(key.to_string(), value.clone());
            found.push(new_group.clone());
        }
    }
    
    // 2. recursively search child groups
    for child in group.values() {
        found.extend(search_in_group(&child.children, key, Some(&new_group)));
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
        let mut watcher: RecommendedWatcher = match RecommendedWatcher::new(tx, notify::Config::default()) {
            Ok(w) => w,
            Err(e) => {
                log::error!("初始化 CSV 监听失败: {e}");
                return;
            }
        };

        if let Err(e) = watcher.watch(std::path::Path::new(&csv_path), RecursiveMode::NonRecursive) {
            log::error!("监听 CSV 文件失败（{}）: {e}", csv_path);
            return;
        }

        for res in rx {
            match res {
                Ok(event) if event.kind.is_modify() => {
                    for path in &event.paths {
                        if let Some(p) = path.to_str() {
                            log::info!("检测到 CSV 更新: {:?}", p);
                            // 重新处理 CSV 并更新缓存
                            let config = match GLOBAL_CONFIG.read() {
                                Ok(cfg) => cfg.clone(),
                                Err(e) => {
                                    log::error!("读取配置失败（锁已污染），跳过本次重载: {e}");
                                    continue;
                                }
                            };

                            match DataProcessor::new(config).process_csv_file(p) {
                                Ok(result) => {
                                    match GLOBAL_CACHE.write() {
                                        Ok(mut cache) => {
                                            *cache = result;
                                            log::info!("CSV 缓存已更新");
                                        }
                                        Err(e) => {
                                            log::error!("更新缓存失败（锁已污染）: {e}");
                                        }
                                    }
                                }
                                Err(err) => {
                                    log::error!("CSV 重新处理失败（{}）: {err}", p);
                                }
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
    let mut watcher: RecommendedWatcher = match RecommendedWatcher::new(tx, notify::Config::default()) {
        Ok(w) => w,
        Err(e) => {
            log::error!("初始化配置文件监听失败: {e}");
            return;
        }
    };

    if let Err(e) = watcher.watch(std::path::Path::new("config/groups.toml"), RecursiveMode::NonRecursive) {
        log::error!("监听配置文件失败: {e}");
        return;
    }

    std::thread::spawn(move || {
        for res in rx {
            match res {
                Ok(event) if event.kind.is_modify() => {
                    for path in &event.paths {
                        if path.ends_with("groups.toml") && event.kind.is_modify() {
                            // Load the new configuration
                            match GroupsConfig::load_from_file("config/groups.toml") {
                                Ok(new_cfg) => match GLOBAL_CONFIG.write() {
                                    Ok(mut cfg) => {
                                        *cfg = new_cfg;
                                        log::info!("配置已重新加载");
                                    }
                                    Err(e) => {
                                        log::error!("写入新配置失败（锁已污染）: {e}");
                                    }
                                },
                                Err(err) => {
                                    log::error!("重新加载配置失败: {err}");
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    log::error!("配置文件监听错误: {err}");
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
