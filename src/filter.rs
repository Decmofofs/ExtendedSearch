use regex::Regex;
use chrono::{Local, TimeZone};

/// 日期限制类型
#[derive(Debug, Clone, PartialEq)]
pub enum DateLimitType {
    None,
    Days(i32, bool),  // 值, whether_new(true=内/newer, false=外/older)
    Weeks(i32, bool), // 值, whether_new(true=内/newer, false=外/older)
    Years(i32, bool), // 值, whether_new(true=内/newer, false=外/older)
    Specific {
        // 起始日期
        minimum_year: i32,
        minimum_month: u32,
        minimum_day: u32,
        // 结束日期
        maximum_year: i32,
        maximum_month: u32,
        maximum_day: u32,
    },
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
    
    /// 完整指定日期模式的起始日期
    pub start_year: Option<i32>,
    pub start_month: Option<i32>,
    pub start_day: Option<i32>,
    /// 完整指定日期模式的结束日期
    pub end_year: Option<i32>,
    pub end_month: Option<i32>,
    pub end_day: Option<i32>,
    
    /// 快速选择模式相关
    pub time_newer: Option<bool>, // true=内, false=外
    
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
            start_year: None,
            start_month: None,
            start_day: None,
            end_year: None,
            end_month: None,
            end_day: None,
            time_newer: Some(true),
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
        time_newer: bool, // 新增：是否"内"(true)或"外"(false)
        end_year: i32, // 新增：完整日期的结束年
        end_month: i32, // 新增：完整日期的结束月
        end_day: i32, // 新增：完整日期的结束日
    ) -> Result<Self, Box<dyn std::error::Error>> {
        println!("from_ui_data: date_limit_type={}, time_newer={}", date_limit_type, time_newer);
        // 转换文件大小从MB到字节
        let min_file_size = (min_file_size_mb as u64) * 1024 * 1024;
        let max_file_size = (max_file_size_mb as u64) * 1024 * 1024;
          // 转换日期限制
        let date_limit = match date_limit_type {
            0 => DateLimitType::None,
            1 => DateLimitType::Days(date_limit_value, time_newer), // 使用time_newer参数
            2 => DateLimitType::Weeks(date_limit_value, time_newer), // 使用time_newer参数
            3 => DateLimitType::Years(date_limit_value, time_newer), // 使用time_newer参数
            4 => DateLimitType::Specific {
                minimum_year: specific_year,
                minimum_month: specific_month as u32,
                minimum_day: specific_day as u32,
                maximum_year: end_year, // 使用end_year而不是与起始日期相同
                maximum_month: end_month as u32, // 使用end_month
                maximum_day: end_day as u32, // 使用end_day
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
            start_year: None, // 在调用site提供额外参数
            start_month: None,
            start_day: None,
            end_year: None,
            end_month: None,
            end_day: None,
            time_newer: Some(true), // 默认为"内"
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
                    println!("应用无时间限制设置");
                },
                DateLimitType::Days(days, whether_new) => {
                    crate::search_file::settings::SEARCH_TIMELIMIT = true;
                    crate::search_file::settings::SEARCH_TIMELIMIT_CURRENTTIME_TYPE = true;
                    crate::search_file::settings::SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER = *whether_new;
                    crate::search_file::settings::SEARCH_COMPARE_WITH_CURRENT_TIME_LIMIT = (*days as u64) * 24 * 60 * 60;
                    println!("应用天数限制：{}天{}，设置SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER={}", 
                        days, if *whether_new { "内" } else { "外" }, *whether_new);
                },
                DateLimitType::Weeks(weeks, whether_new) => {
                    crate::search_file::settings::SEARCH_TIMELIMIT = true;
                    crate::search_file::settings::SEARCH_TIMELIMIT_CURRENTTIME_TYPE = true;
                    crate::search_file::settings::SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER = *whether_new;
                    crate::search_file::settings::SEARCH_COMPARE_WITH_CURRENT_TIME_LIMIT = (*weeks as u64) * 7 * 24 * 60 * 60;
                    println!("应用周数限制：{}周{}，设置SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER={}", 
                        weeks, if *whether_new { "内" } else { "外" }, *whether_new);
                },
                DateLimitType::Years(years, whether_new) => {
                    crate::search_file::settings::SEARCH_TIMELIMIT = true;
                    crate::search_file::settings::SEARCH_TIMELIMIT_CURRENTTIME_TYPE = true;
                    crate::search_file::settings::SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER = *whether_new;
                    crate::search_file::settings::SEARCH_COMPARE_WITH_CURRENT_TIME_LIMIT = (*years as u64) * 365 * 24 * 60 * 60;
                    println!("应用年数限制：{}年{}，设置SEARCH_COMPARE_WITH_CURRENT_TIME_NEWER={}", 
                        years, if *whether_new { "内" } else { "外" }, *whether_new);
                },
                DateLimitType::Specific { 
                    minimum_year, minimum_month, minimum_day,
                    maximum_year, maximum_month, maximum_day 
                } => {
                    crate::search_file::settings::SEARCH_TIMELIMIT = true;
                    crate::search_file::settings::SEARCH_TIMELIMIT_CURRENTTIME_TYPE = false;
                    
                    // 使用 settings.rs 中的 add_type2_timelimit 函数来设置日期范围
                    println!("应用完整日期范围限制：从 {}/{}/{} 到 {}/{}/{}", 
                        minimum_year, minimum_month, minimum_day,
                        maximum_year, maximum_month, maximum_day);
                        
                    crate::search_file::settings::add_type2_timelimit(
                        *minimum_year, *minimum_month, *minimum_day,
                        *maximum_year, *maximum_month, *maximum_day
                    );
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

