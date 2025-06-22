pub const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;
pub const FILE_ATTRIBUTE_READONLY: u32 = 0x00000001;
pub const FILE_ATTRIBUTE_SYSTEM: u32 = 0x00000004;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{NaiveDate, Datelike};

pub static mut SEARCH_HIDDEN_FILES: bool = true; // Set to true if you want to search hidden files
pub static mut SEARCH_HIDDEN_FOLDERS: bool = false; // Set to true if you want to search hidden folders
pub static mut SEARCH_FILESIZE_MAXIMUM_LIMIT: u64 = 1024 * 1024 * 1024; // Set the maximum file size for searching
pub static mut SEARCH_FILESIZE_MINIMUM_LIMIT: u64 = 0; // Set the minimum file size for searching
pub static mut SEARCH_READONLY: bool = false; // Set to true if you want to search read-only files
pub static mut SAVE_HASH: bool = false; // Set to true if you want to save file hashes
pub static mut REGEX_CONTAIN_PATH: bool = false;// regex匹配项是否包含路径，即正则项是整个路径的正则项还是仅文件名的正则项
pub static mut SEARCH_SYSTEM_FILES: bool = false; // Set to true if you want to search system files
pub static mut SEARCH_TIMELIMIT: bool = false;
pub static mut SEARCH_TIMELIMIT_CURRENTTIME_TYPE: bool = false;
pub static mut SEARCH_COMPARE_WITH_CURRENT_TIME_LIMIT: u64 = 0;
pub static mut SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER: bool = true;
pub static mut SEARCH_MODIFY_TIME_MAXIMUM_LIMIT: u64 = 1145141919810; // Set the maximum modification time for searching (0 means no limit)
pub static mut SEARCH_MODIFY_TIME_MINIMUM_LIMIT: u64 = 0; // Set the minimum modification time for searching (0 means no limit)
pub static mut SEARCH_DEPTH: u8 = 255; // Default search depth

use std::io::Read;

// 从config.json文件加载设置
pub fn load_settings() -> (u8, String) {
    let mut search_depth: u8 = 255; // Default search depth
    let mut file_regex: String = String::from(r".*"); // Default regex
    let open_file = std::fs::File::open("config.json");
    match open_file {
        Err(e) => {
            eprintln!("Error opening config file: {}", e);
            return (search_depth, file_regex);
        },
        Ok(mut file) => {
            // Load settings from the file
            let mut config_contents = String::new();
            file.read_to_string(&mut config_contents).expect("Failed to read config file");
            let json: serde_json::Value = serde_json::from_str(&config_contents).unwrap();
            unsafe {
                SEARCH_HIDDEN_FILES = json.get("search_hidden_files").unwrap().as_bool().unwrap();
                SEARCH_HIDDEN_FOLDERS = json.get("search_hidden_folders").unwrap().as_bool().unwrap();
                SEARCH_FILESIZE_MAXIMUM_LIMIT = json.get("search_filesize_maximum_limit").unwrap().as_u64().unwrap();
                SEARCH_FILESIZE_MINIMUM_LIMIT = json.get("search_filesize_minimum_limit").unwrap().as_u64().unwrap();
                SEARCH_READONLY = json.get("search_readonly").unwrap().as_bool().unwrap();
                SAVE_HASH = json.get("save_hash").unwrap().as_bool().unwrap();
                REGEX_CONTAIN_PATH = json.get("regex_contain_path").unwrap().as_bool().unwrap();
                SEARCH_SYSTEM_FILES = json.get("search_system_files").unwrap().as_bool().unwrap();
                SEARCH_TIMELIMIT = json.get("search_timelimit").unwrap().as_bool().unwrap();
                SEARCH_TIMELIMIT_CURRENTTIME_TYPE = json.get("search_timelimit_currenttime_type").unwrap().as_bool().unwrap();
                SEARCH_COMPARE_WITH_CURRENT_TIME_LIMIT = json.get("search_compare_with_current_time_limit").unwrap().as_u64().unwrap();
                SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER = json.get("search_compare_with_current_time_newer").unwrap().as_bool().unwrap();
                SEARCH_MODIFY_TIME_MAXIMUM_LIMIT = json.get("search_modify_time_maximum_limit").unwrap().as_u64().unwrap();
                SEARCH_MODIFY_TIME_MINIMUM_LIMIT = json.get("search_modify_time_minimum_limit").unwrap().as_u64().unwrap();
                search_depth = json.get("search_depth").unwrap().as_u64().unwrap() as u8;
                file_regex = json.get("file_regex").unwrap().as_str().unwrap().to_string();
            }
        }
    }
    (search_depth, file_regex)
}

// 保存设置到config.json文件
pub fn save_settings(search_depth:u8 , file_regex: &String) {
    let settings = serde_json::json!({
        "search_hidden_files": unsafe { SEARCH_HIDDEN_FILES },
        "search_hidden_folders": unsafe { SEARCH_HIDDEN_FOLDERS },
        "search_filesize_maximum_limit": unsafe { SEARCH_FILESIZE_MAXIMUM_LIMIT },
        "search_filesize_minimum_limit": unsafe { SEARCH_FILESIZE_MINIMUM_LIMIT },
        "search_readonly": unsafe { SEARCH_READONLY },
        "save_hash": unsafe { SAVE_HASH },
        "regex_contain_path": unsafe { REGEX_CONTAIN_PATH },
        "search_system_files": unsafe { SEARCH_SYSTEM_FILES },
        "search_timelimit": unsafe { SEARCH_TIMELIMIT },
        "search_timelimit_currenttime_type": unsafe { SEARCH_TIMELIMIT_CURRENTTIME_TYPE },
        "search_compare_with_current_time_limit": unsafe { SEARCH_COMPARE_WITH_CURRENT_TIME_LIMIT },
        "search_compare_with_current_time_newer": unsafe { SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER },
        "search_modify_time_maximum_limit": unsafe { SEARCH_MODIFY_TIME_MAXIMUM_LIMIT },
        "search_modify_time_minimum_limit": unsafe { SEARCH_MODIFY_TIME_MINIMUM_LIMIT },
        "search_depth": search_depth,
        "file_regex": file_regex
    });

    let json_string = serde_json::to_string(&settings).unwrap();
    std::fs::write("config.json", json_string).expect("Unable to write config file");
}

// 下面的函数用于其他程序调用以修改参数
// 对深度和正则表达式的修改方式是直接修改传入search的变量

// 设置是否搜索隐藏文件
pub fn modify_search_hidden_files(value: bool) {
    unsafe {
        SEARCH_HIDDEN_FILES = value;
    }
}

// 设置是否搜索隐藏文件夹
pub fn modify_search_hidden_folders(value: bool) {
    unsafe {
        SEARCH_HIDDEN_FOLDERS = value;
    }
}

// 设置搜索文件大小的最大限制
pub fn modify_search_filesize_maximum_limit(value: u64) {
    unsafe {
        SEARCH_FILESIZE_MAXIMUM_LIMIT = value;
    }
}

// 设置搜索文件大小的最小限制
pub fn modify_search_filesize_minimum_limit(value: u64) {
    unsafe {
        SEARCH_FILESIZE_MINIMUM_LIMIT = value;
    }
}

// 设置是否搜索只读文件
pub fn modify_search_readonly(value: bool) {
    unsafe {
        SEARCH_READONLY = value;
    }
}

// 设置是否保存文件哈希，用于去重
pub fn modify_save_hash(value: bool) {
    unsafe {
        SAVE_HASH = value;
    }
}

// 设置正则表达式匹配的是文件名还是文件路径
pub fn modify_regex_contain_path(value: bool) {
    unsafe {
        REGEX_CONTAIN_PATH = value;
    }
}

// 设置是否搜索系统文件
pub fn modify_search_system_files(value: bool) {
    unsafe {
        SEARCH_SYSTEM_FILES = value;
    }
}

// 设置时间限制
// 时间限制有两种:第一种是当前时间到现在的时间限制，第二种是修改时间的最大最小限制

// 取消时间限制
pub fn cancel_file_timelimit() {
    unsafe {
        SEARCH_TIMELIMIT = false;
    }
}

// 传入当前时间到现在的时间限制 数字 + 单位 + 修改日期比限制时间更新/更旧
pub fn add_type1_timelimit(num : u64, unit: &str,newer: bool) {
    let to_now_limit: u64 = match unit {
        "second" => num,
        "minute" => num * 60,
        "hour" => num * 3600,
        "day" => num * 86400,
        "week" => num * 604800,
        "month" => num * 2592000,
        "year" => num * 31536000,
        _ => 0,
    };
    unsafe {
        SEARCH_TIMELIMIT = true;
        SEARCH_TIMELIMIT_CURRENTTIME_TYPE = true;
        SEARCH_COMPARE_WITH_CURRENT_TIME_LIMIT = to_now_limit;
        SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER = newer;
    }
}

// 传入修改日期 下限的年月日 + 上限的年月日
pub fn add_type2_timelimit(minimum_year: i32, minimum_month: u32, minimum_day: u32, maximum_year: i32, maximum_month: u32, maximum_day: u32) {
    let minimum_date = NaiveDate::from_ymd_opt(minimum_year, minimum_month, minimum_day);
    let maximum_date = NaiveDate::from_ymd_opt(maximum_year, maximum_month, maximum_day);

    if let (Some(min_date), Some(max_date)) = (minimum_date, maximum_date) {
        
        let minimum_timestamp = min_date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as u64;
        let maximum_timestamp = max_date.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp() as u64;
        unsafe {
            SEARCH_TIMELIMIT = true;
            SEARCH_TIMELIMIT_CURRENTTIME_TYPE = false;
            SEARCH_MODIFY_TIME_MINIMUM_LIMIT = minimum_timestamp;
            SEARCH_MODIFY_TIME_MAXIMUM_LIMIT = maximum_timestamp;
        }
    } else {
        eprintln!("Invalid date input: minimum ({}, {}, {}), maximum ({}, {}, {})",
            minimum_year, minimum_month, minimum_day,
            maximum_year, maximum_month, maximum_day);
    }
}