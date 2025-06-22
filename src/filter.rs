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
    Months(i32, bool), // 新增：月份限制，值, whether_new(true=内/newer, false=外/older)
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
            5 => DateLimitType::Months(date_limit_value, time_newer), // 新增：月份限制
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
    
    
    
}

