use crate::filter::SearchFilter;
use crate::search_file::settings;
use std::path::PathBuf;
use chrono::Datelike;

/// 帮助将UI设置应用到search_file模块的settings
pub struct SearchHelper;

impl SearchHelper {
    /// 应用过滤器设置到search_file模块
    pub fn apply_filter_settings(filter: &SearchFilter) {
        // 使用settings.rs中定义的函数
        
        // 隐藏文件和目录设置
        settings::modify_search_hidden_files(filter.search_hidden_files);
        settings::modify_search_hidden_folders(filter.search_hidden_folders);
        
        // 只读文件设置
        settings::modify_search_readonly(filter.search_readonly_files);
        
        // 系统文件设置
        settings::modify_search_system_files(true); // 默认搜索系统文件
        
        // 文件大小限制
        settings::modify_search_filesize_minimum_limit(filter.min_file_size);
        settings::modify_search_filesize_maximum_limit(filter.max_file_size);
        
        // 记录哈希值设置
        settings::modify_save_hash(filter.record_hash);
        
        // 正则表达式设置
        settings::modify_regex_contain_path(match filter.regex_target {
            crate::filter::RegexTarget::FileName => false,
            crate::filter::RegexTarget::FilePath => true,
        });
        
        // 时间限制设置
        match &filter.date_limit {
            crate::filter::DateLimitType::None => {
                // 取消时间限制
                settings::cancel_file_timelimit();
            },
            crate::filter::DateLimitType::Days(days) => {
                // 使用天数限制
                settings::add_type1_timelimit(*days as u64, "day", true);
            },
            crate::filter::DateLimitType::Weeks(weeks) => {
                // 使用周数限制
                settings::add_type1_timelimit(*weeks as u64, "week", true);
            },
            crate::filter::DateLimitType::Years(years) => {
                // 使用年数限制
                settings::add_type1_timelimit(*years as u64, "year", true);
            },
            crate::filter::DateLimitType::Specific { year, month, day } => {
                // 使用具体日期限制 - 从当前日期到指定日期
                let today = chrono::Local::now().date_naive();
                
                // 使用add_type2_timelimit函数，指定日期范围
                settings::add_type2_timelimit(
                    today.year(), 
                    today.month(), 
                    today.day(),
                    *year, 
                    *month, 
                    *day
                );
            },
        }
    }
    
    /// 执行搜索
    /// 
    /// # Arguments
    /// 
    /// * `paths` - 要搜索的路径列表
    /// * `search_depth` - 搜索深度
    /// * `regex_pattern` - 正则表达式模式
    /// * `filter` - 过滤器设置
    /// 
    /// # Returns
    /// 
    /// 搜索结果列表
    pub fn perform_search(
        paths: &[PathBuf], 
        search_depth: u8, 
        regex_pattern: &str,
        filter: &SearchFilter
    ) -> Vec<crate::search_file::SingleFileInformations> {
        // 先应用过滤器设置
        Self::apply_filter_settings(filter);
        
        // 执行搜索
        crate::search_file::get_files(paths, search_depth, &regex_pattern.to_string())
    }
    
    /// 去重搜索结果
    pub fn remove_duplicates(
        files: &mut Vec<crate::search_file::SingleFileInformations>
    ) {
        crate::search_file::unique_files(files);
    }
    
    /// 排序搜索结果
    pub fn sort_results(
        files: &mut Vec<crate::search_file::SingleFileInformations>,
        sort_type: &str,
        reversed: bool
    ) {
        crate::search_file::sort_files(files, sort_type.to_string(), reversed);
    }
    
    /// 导出搜索结果
    pub fn export_results(
        files: &[crate::search_file::SingleFileInformations],
        file_path: Option<&str>
    ) -> Result<(), std::io::Error> {
        if let Some(path) = file_path {
            // 导出到指定文件
            let json_str = serde_json::to_string(files)?;
            std::fs::write(path, json_str)?;
            Ok(())
        } else {
            // 使用默认文件名
            crate::search_file::export_found_files(files);
            Ok(())
        }
    }
    
    /// 复制文件到目标目录
    pub fn copy_files_to(
        files: &[crate::search_file::SingleFileInformations],
        destination: &PathBuf
    ) -> std::io::Result<()> {
        crate::search_file::copy_files(files, destination)
    }
    
    /// 移动文件到目标目录
    pub fn move_files_to(
        files: &[crate::search_file::SingleFileInformations],
        destination: &PathBuf
    ) -> std::io::Result<()> {
        crate::search_file::move_files(files, destination)
    }
    
    /// 删除文件
    pub fn delete_files(
        files: &[crate::search_file::SingleFileInformations]
    ) -> std::io::Result<()> {
        crate::search_file::delete_files(files)
    }
    
    /// 映射文件（保持目录结构）
    pub fn map_files(
        files: &[crate::search_file::SingleFileInformations],
        source: &PathBuf,
        destination: &PathBuf
    ) {
        crate::search_file::mapping_files(files, source, destination)
    }
}
