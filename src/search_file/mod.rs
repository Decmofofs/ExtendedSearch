use std::arch::global_asm;
use std::os::windows::fs::MetadataExt;
use std::string::String;
use std::vec::Vec;
use std::time::SystemTime as Time;
use std::hash::Hash;
use std::hash::Hasher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::path::Path;
use std::fs;
use std::thread;
use std::fs::DirEntry;
use sha2::{Sha256, Digest};
use regex::Regex;
use std::time::Duration;
use std::time::UNIX_EPOCH;
use serde::Serialize;
use serde::Deserialize;
use trash;


pub mod settings;
pub mod build_tree;

#[derive(Serialize, Deserialize)]
pub struct SingleFileInformations {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub time: u64,
    pub hash: String
}


fn is_hidden(entry: &DirEntry) -> bool {
    entry
         .file_name()
         .to_str()
         .map(|s| {s.starts_with(".")}) 
         .unwrap_or(false)
}

// search的内部实现，无需手动调用
fn search_in_path( filter : fn( metadata:&fs::Metadata )->bool ,curpath: &PathBuf ,search_depth:u8 , file_regex: &String) -> Vec<SingleFileInformations> {
    let mut result = Vec::new();
    let mut handles = Vec::new();
    for entry in fs::read_dir(curpath).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        // println!("Searching: {:?}", entry.file_name());

        // If it's a directory, recurse into it
        if path.is_dir() {

            // Skip directories if we've reached the max depth
            if search_depth == 1 {
                continue; 
            }

            // Skip hidden directories
            if unsafe{!settings::SEARCH_HIDDEN_FOLDERS} {
                if is_hidden(&entry) {
                    continue; 
                }
            }

            // use multithreading to search in subdirectories
            let file_regex_clone = file_regex.clone();
            let handle = thread::spawn(move || {
                search_in_path(filter, &path, search_depth - 1, &file_regex_clone)
            });
            handles.push(handle);

        } else {
            let metadata = fs::metadata(&path).unwrap();
            if metadata.is_file() {

                // 正则表达式匹配文件名
                {
                    let file_regex = file_regex.clone();
                    if !file_regex.is_empty() {
                        let regex = Regex::new(&file_regex).unwrap();
                        if unsafe{settings::REGEX_CONTAIN_PATH} {
                            if let Some(file_name) = path.to_str() {
                                println!("Full path: {}", file_name);
                                if !regex.is_match(&file_name) {
                                    continue; // 文件名不匹配正则表达式
                                }
                            }
                        } else {
                            if let Some(file_name) = entry.file_name().to_str() {
                                if !regex.is_match(file_name) {
                                    continue; // 文件名不匹配正则表达式
                                }
                            }
                        }
                    }
                }

                if filter(&metadata) {
                    
                    let hash_value:String;

                    if unsafe{settings::SAVE_HASH} {
                        let mut file = fs::File::open(&entry.path()).unwrap();
                        let mut hasher = Sha256::new();
                        let mut buffer = [0; 1024];
                        while let Ok(bytes_read) = file.read(&mut buffer) {
                            if bytes_read == 0 {
                                break;
                            }
                            hasher.update(&buffer[..bytes_read]);
                        }
                        let hash = hasher.finalize();
                        hash_value = format!("{:x}", hash);
                    } else {
                        hash_value = String::new();
                    }

                    let file_info = SingleFileInformations {
                        path,
                        name: entry.file_name().into_string().unwrap(),
                        size: metadata.len(),
                        time: metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        hash: hash_value,
                    };

                    result.push(file_info);
                    
                }
            }
        }
    }
    for handle in handles {
        result.extend(handle.join().unwrap());
    }
    result
}

fn filter(metadata: &fs::Metadata) -> bool {
    // path.extension().map_or(false, |ext| ext == "rs")
    // println!("Checking: {:?}", path);
    // println!("{:?}",metadata);

    // 检查是否跳过隐藏文件
    if unsafe{!settings::SEARCH_HIDDEN_FILES} {
        if metadata.file_attributes() & settings::FILE_ATTRIBUTE_HIDDEN != 0 {
            return false; // Skip hidden files
        }
    }

    // 检查文件大小限制
    {
        let file_size: u64 = metadata.len();
        if unsafe{file_size > settings::SEARCH_FILESIZE_MAXIMUM_LIMIT} {
            return false; // Skip files larger than the maximum limit
        }
        if unsafe{file_size < settings::SEARCH_FILESIZE_MINIMUM_LIMIT} {
            return false; // Skip files smaller than the minimum limit
        }
    }

    // 检查是否跳过只读文件
    // 只读文件的属性是 FILE_ATTRIBUTE_READONLY
    if unsafe{!settings::SEARCH_READONLY} {
        if metadata.file_attributes() & settings::FILE_ATTRIBUTE_READONLY != 0 {
            return false; // Skip read-only files
        }
    }

    // 检查文件日期
    if unsafe{settings::SEARCH_TIMELIMIT} {
        if unsafe{settings::SEARCH_TIMELIMIT_CURRENTTIME_TYPE} {
            if let Ok(modified) = metadata.modified() {
                // 若比指定时间更新且查询的是旧文件则return
                // 若比指定时间早且查询的是新文件则return
                if (modified > Time::now() - Duration::from_secs(unsafe{settings::SEARCH_COMPARE_WITH_CURRENT_TIME_LIMIT})) ^ unsafe{settings::SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER} {
                    return false; // Skip files older than the specified limit
                }
            }
        } else {
            let modified = metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs();
            if modified > unsafe{settings::SEARCH_MODIFY_TIME_MAXIMUM_LIMIT} {
                return false;
            }
            if modified < unsafe{settings::SEARCH_MODIFY_TIME_MINIMUM_LIMIT} {
                return false;
            }
        }
    }

    // 检查是否跳过系统文件
    if unsafe{!settings::SEARCH_SYSTEM_FILES} {
        if metadata.file_attributes() & settings::FILE_ATTRIBUTE_SYSTEM != 0 {
            return false; // Skip system files
        }
    }

    true
}

// This function searches for files in the given paths with the specified search depth and file regex.
// It returns a vector of SingleFileInformations containing the file details.
// 需要传入的参数: 向下搜索的深度search_depth , 正则表达式file_regex , 其他的参数通过settings.rs中的静态变量设置
pub fn get_files(paths:&[PathBuf],search_depth:u8,file_regex: &String) -> Vec<SingleFileInformations> {
    let mut all_found_files: Vec<SingleFileInformations> = Vec::new();
    for current_path in paths {
        all_found_files.extend(search_in_path(filter, &current_path , search_depth, &file_regex));
    }
    // 这里可以添加获取文件的逻辑
    all_found_files
}

// This function sorts the files based on selected sort type.
// sort_type can be "name", "size", "time", or "path".
// If reversed is true, it will sort in descending order.
// 在整体代码中的行为:传入get_files或unique_files得到的Vec,将其按照指定方式排序
pub fn sort_files(files: &mut Vec<SingleFileInformations>, sort_type: String, reversed: bool) {
    match sort_type.as_str() {
        "name" => files.sort_by(|a, b| a.name.cmp(&b.name)),
        "size" => files.sort_by(|a, b| a.size.cmp(&b.size)),
        "time" => files.sort_by(|a, b| a.time.cmp(&b.time)),
        "path" => files.sort_by(|a, b| a.path.cmp(&b.path)),
        _ => (),
    }
    if reversed {
        files.reverse();
    }
}

// This function removes duplicate files based on their hash.
// It sorts the files by hash and then removes duplicates.
// 整体代码中的行为:若选择使用SHA256哈希值来去重,則在get_files後調用此函數作為新的Vec
pub fn unique_files(files: &mut Vec<SingleFileInformations>) {
    if files.is_empty() || files[0].hash.is_empty() {
        return;
    }
    files.sort_by(|a, b| a.hash.cmp(&b.hash));
    files.dedup_by(|a, b| a.hash == b.hash);
}

// This function reports the found files to a JSON file named "search_result.json".
// 整体代码中的行为:点按导出结果按钮后将搜索结果写入search_result.json文件
pub fn export_found_files(found_files: &[SingleFileInformations]) {
    let json_str = serde_json::to_string(found_files).unwrap();
    std::fs::write("search_result.json", json_str).expect("Unable to write result to file");
}

// This function copies files in the list to the destination directory.
// It will overwrite files in the destination if they already exist.
// 行为：点按复制按钮后将搜索结果复制到指定目录，调用时要用 &path::absolute("地址").unwrap().to_path_buf() 这种写法
pub fn copy_files(files: &[SingleFileInformations], destination: &PathBuf) -> io::Result<()> {
    for file in files {
        let dest_path = destination.join(&file.name);
        fs::copy(&file.path, dest_path)?;
    }
    Ok(())
}

// This function moves files in the list to the destination directory.
// It will overwrite files in the destination if they already exist.
// 行为: 点按移动按钮后将搜索结果移动到指定目录，同上（覆盖）
pub fn move_files(files: &[SingleFileInformations], destination: &PathBuf) -> io::Result<()> {
    for file in files {
        let dest_path = destination.join(&file.name);
        fs::rename(&file.path, dest_path)?;
    }
    Ok(())
}

// delete files in the list
// This function will remove the files from the filesystem.
// It does not check if the files exist, so it may return an error if a file does not exist.
// 行为: 删除，后略
pub fn delete_files(files: &[SingleFileInformations]) -> io::Result<()> {
    for file in files {
        // fs::remove_file(&file.path)?;
        trash::delete(&file.path).expect("Failed to delete file");
    }
    Ok(())
}

// This function copies files in the list from the source directory to the destination directory.
// It can maintain the original directory structure. 
// 行为: 维持原有目录结构的复制，点按映射按钮后将搜索结果复制到指定目录
// 将搜索到的文件在source中的部分全部维持原文件夹架构复制到destination (由于有多个搜索源故需指定source防止重名文件冲突)
// 注: 传入的source和destination必须是绝对路径,files如果按照上面的实现的话内部的path也是绝对路径
pub fn mapping_files(files: &[SingleFileInformations], source: &PathBuf, destination: &PathBuf) {
    for file in files {
        let relative_path = match file.path.strip_prefix(source) {
            Ok(relative) => relative.to_path_buf(),
            Err(_) => {
                continue; // Skip this file if it doesn't match the source path
            }
        };
        let dest_path = destination.join(relative_path);
        
        // Create the parent directory if it doesn't exist
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directory");
        }
        
        // Copy the file
        fs::copy(&file.path, dest_path).expect("Failed to copy file");
    }
}