use std::time::SystemTime;
use std::path::Path;
use regex::Regex;
use chrono::{DateTime, Local, TimeZone};

/// 日期限制类型
#[derive(Debug, Clone, PartialEq)]
pub enum DateLimitType {
    None,
    Days(i32),
    Weeks(i32),
    Years(i32),
    Specific { year: i32, month: u32, day: u32 },
}

/// 正则表达式匹配目标
#[derive(Debug, Clone, PartialEq)]
pub enum RegexTarget {
    FileName,
    FilePath,
}

/// 搜索过滤条件
#[derive(Debug, Clone)]
pub struct SearchFilter {
    /// 是否搜索隐藏文件
    pub search_hidden_files: bool,
    /// 是否搜索隐藏文件夹
    pub search_hidden_folders: bool,
    /// 是否搜索只读文件
    pub search_readonly_files: bool,
    /// 文件大小限制 (单位: 字节)
    pub min_file_size: u64,
    pub max_file_size: u64,
    /// 日期限制
    pub date_limit: DateLimitType,
    /// 正则表达式模式
    pub regex_pattern: Option<Regex>,
    pub regex_target: RegexTarget,
    /// 是否记录哈希值
    pub record_hash: bool,
}

impl Default for SearchFilter {
    fn default() -> Self {
        Self {
            search_hidden_files: true,
            search_hidden_folders: true,
            search_readonly_files: true,
            min_file_size: 0,
            max_file_size: 64 * 1024 * 1024 * 1024, // 64MB in bytes
            date_limit: DateLimitType::None,
            regex_pattern: None,
            regex_target: RegexTarget::FileName,
            record_hash: false,
        }
    }
}

impl SearchFilter {
    /// 从UI数据创建搜索过滤器
    pub fn from_ui_data(
        search_hidden_files: bool,
        search_hidden_folders: bool,
        search_readonly_files: bool,
        min_file_size_mb: i32,
        max_file_size_mb: i32,
        date_limit_type: i32,
        date_limit_value: i32,
        specific_year: i32,
        specific_month: i32,
        specific_day: i32,
        regex_pattern: &str,
        regex_target: i32,
        record_hash: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // 转换文件大小从MB到字节
        let min_file_size = (min_file_size_mb as u64) * 1024 * 1024;
        let max_file_size = (max_file_size_mb as u64) * 1024 * 1024;
        
        // 转换日期限制
        let date_limit = match date_limit_type {
            0 => DateLimitType::None,
            1 => DateLimitType::Days(date_limit_value),
            2 => DateLimitType::Weeks(date_limit_value),
            3 => DateLimitType::Years(date_limit_value),
            4 => DateLimitType::Specific {
                year: specific_year,
                month: specific_month as u32,
                day: specific_day as u32,
            },
            _ => DateLimitType::None,
        };
        
        // 转换正则表达式
        let regex_pattern = if regex_pattern.trim().is_empty() {
            None
        } else {
            Some(Regex::new(regex_pattern)?)
        };
        
        // 转换正则表达式目标
        let regex_target = match regex_target {
            0 => RegexTarget::FileName,
            _ => RegexTarget::FilePath,
        };
        
        Ok(Self {
            search_hidden_files,
            search_hidden_folders,
            search_readonly_files,
            min_file_size,
            max_file_size,
            date_limit,
            regex_pattern,
            regex_target,
            record_hash,
        })
    }
    
    /// 将过滤器设置应用到搜索模块的全局变量
    pub fn apply_to_settings(&self) {
        unsafe {
            // 设置隐藏文件和文件夹的搜索选项
            crate::search_file::settings::SEARCH_HIDDEN_FILES = self.search_hidden_files;
            crate::search_file::settings::SEARCH_HIDDEN_FOLDERS = self.search_hidden_folders;
            
            // 设置只读文件搜索选项
            crate::search_file::settings::SEARCH_READONLY = self.search_readonly_files;
            
            // 设置系统文件搜索选项 (默认打开)
            crate::search_file::settings::SEARCH_SYSTEM_FILES = true;
            
            // 设置文件大小限制
            crate::search_file::settings::SEARCH_FILESIZE_MINIMUM_LIMIT = self.min_file_size;
            crate::search_file::settings::SEARCH_FILESIZE_MAXIMUM_LIMIT = self.max_file_size;
            
            // 设置时间限制
            match &self.date_limit {
                DateLimitType::None => {
                    crate::search_file::settings::SEARCH_TIMELIMIT = false;
                },
                DateLimitType::Days(days) => {
                    crate::search_file::settings::SEARCH_TIMELIMIT = true;
                    crate::search_file::settings::SEARCH_TIMELIMIT_CURRENTTIME_TYPE = true;
                    crate::search_file::settings::SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER = false;
                    crate::search_file::settings::SEARCH_COMPARE_WITH_CURRENT_TIME_LIMIT = (*days as u64) * 24 * 60 * 60;
                },
                DateLimitType::Weeks(weeks) => {
                    crate::search_file::settings::SEARCH_TIMELIMIT = true;
                    crate::search_file::settings::SEARCH_TIMELIMIT_CURRENTTIME_TYPE = true;
                    crate::search_file::settings::SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER = false;
                    crate::search_file::settings::SEARCH_COMPARE_WITH_CURRENT_TIME_LIMIT = (*weeks as u64) * 7 * 24 * 60 * 60;
                },
                DateLimitType::Years(years) => {
                    crate::search_file::settings::SEARCH_TIMELIMIT = true;
                    crate::search_file::settings::SEARCH_TIMELIMIT_CURRENTTIME_TYPE = true;
                    crate::search_file::settings::SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER = false;
                    crate::search_file::settings::SEARCH_COMPARE_WITH_CURRENT_TIME_LIMIT = (*years as u64) * 365 * 24 * 60 * 60;
                },
                DateLimitType::Specific { year, month, day } => {
                    crate::search_file::settings::SEARCH_TIMELIMIT = true;
                    crate::search_file::settings::SEARCH_TIMELIMIT_CURRENTTIME_TYPE = false;
                    
                    // 计算指定日期的UNIX时间戳
                    if let Some(date_time) = Local.with_ymd_and_hms(*year, *month, *day, 0, 0, 0).single() {
                        let timestamp = date_time.timestamp() as u64;
                        crate::search_file::settings::SEARCH_MODIFY_TIME_MINIMUM_LIMIT = timestamp;
                        crate::search_file::settings::SEARCH_MODIFY_TIME_MAXIMUM_LIMIT = timestamp + 24 * 60 * 60;
                    }
                },
            }
            
            // 设置正则表达式目标
            crate::search_file::settings::REGEX_CONTAIN_PATH = match self.regex_target {
                RegexTarget::FileName => false,
                RegexTarget::FilePath => true,
            };
            
            // 设置是否计算哈希
            crate::search_file::settings::SAVE_HASH = self.record_hash;
        }
    }
}

