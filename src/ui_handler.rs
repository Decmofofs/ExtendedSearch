use slint::{Model, VecModel, SharedString};
use std::rc::Rc;
use std::path::PathBuf;
use native_dialog::{FileDialog, MessageDialog, MessageType};
use crate::filter::SearchFilter;
use crate::search_file::SingleFileInformations;
use crate::helper::SearchHelper;

slint::include_modules!();

// 搜索结果模型
pub struct SearchResultModel {
    inner: Rc<VecModel<FileInfo>>,
}

impl SearchResultModel {
    fn new() -> Self {
        Self {
            inner: Rc::new(VecModel::default()),
        }
    }
    
    // 清空结果
    fn clear(&self) {
        while self.inner.row_count() > 0 {
            self.inner.remove(0);
        }
    }
      // 添加搜索结果
    fn add_results(&self, results: &[SingleFileInformations]) {
        for file in results {
            let file_info = FileInfo {
                path: file.path.to_string_lossy().to_string().into(),
                name: file.name.clone().into(),
                size: file.size as i32,
                time: file.time as i32,
                hash: file.hash.clone().into(),
                selected: false, // 默认不选中
            };
            self.inner.push(file_info);
        }
    }
    
    // 获取所有搜索结果
    fn get_all(&self) -> Vec<SingleFileInformations> {
        let mut results = Vec::new();
        for i in 0..self.inner.row_count() {
            if let Some(info) = self.inner.row_data(i) {
                results.push(SingleFileInformations {
                    path: PathBuf::from(info.path.as_str()),
                    name: info.name.as_str().to_string(),
                    size: info.size as u64,
                    time: info.time as u64,
                    hash: info.hash.as_str().to_string(),
                });
            }
        }
        results
    }
}

/// UI交互处理器
pub struct UIHandler {
    pub ui: AppWindow,
    pub directories: Rc<VecModel<DirectoryItem>>,
    pub current_filter: Rc<std::cell::RefCell<SearchFilter>>,
    pub search_results: SearchResultModel,
    pub selected_paths: Rc<VecModel<SharedString>>,
}

impl UIHandler {    /// 创建新的UI处理器
    pub fn new() -> Result<Self, slint::PlatformError> {
        let ui = AppWindow::new()?;
        let directories = Rc::new(VecModel::default());
        let current_filter = Rc::new(std::cell::RefCell::new(SearchFilter::default()));
        
        // 将数据模型绑定到UI
        ui.set_directories(directories.clone().into());
        
        // 创建UIHandler实例
        let handler = Self {
            ui,
            directories,
            current_filter,
            search_results: SearchResultModel::new(),
            selected_paths: Rc::new(VecModel::default()),
        };
        
        // 设置初始过滤器数据到UI
        handler.sync_filter_to_ui();
        
        Ok(handler)
    }
      /// 设置所有UI回调
    pub fn setup_callbacks(&self) {
        self.setup_directory_callbacks();
        self.setup_filter_callbacks();
        self.setup_search_callbacks();
    }
      /// 获取当前的过滤设置
    pub fn get_current_filter(&self) -> std::cell::Ref<SearchFilter> {
        self.current_filter.borrow()
    }
    
    /// 获取当前过滤设置的副本
    pub fn get_current_filter_clone(&self) -> SearchFilter {
        self.current_filter.borrow().clone()
    }
    
    /// 设置目录相关的回调
    fn setup_directory_callbacks(&self) {
        // 添加目录回调
        let ui_weak = self.ui.as_weak();
        let directories_clone = self.directories.clone();
        self.ui.on_add_directory(move || {
            if let Some(ui) = ui_weak.upgrade() {
                Self::add_directory(&directories_clone, &ui);
            }
        });
        
        // 删除选中目录回调
        let ui_weak = self.ui.as_weak();
        let directories_clone = self.directories.clone();
        self.ui.on_remove_selected(move || {
            if let Some(ui) = ui_weak.upgrade() {
                Self::remove_selected_directories(&directories_clone, &ui);
            }
        });
        
        // 目录选择状态切换回调
        let ui_weak = self.ui.as_weak();
        let directories_clone = self.directories.clone();
        self.ui.on_directory_toggled(move |index| {
            if let Some(ui) = ui_weak.upgrade() {
                Self::toggle_directory_selection(&directories_clone, &ui, index);
            }
        });
    }    /// 添加目录
    fn add_directory(directories: &Rc<VecModel<DirectoryItem>>, ui: &AppWindow) {
        // 打开文件夹选择对话框
        match FileDialog::new()
            .show_open_single_dir()
        {
            Ok(Some(folder_path)) => {
                // 检查是否已经添加过这个目录
                let path_str = folder_path.to_string_lossy().to_string();
                let mut already_exists = false;
                
                for i in 0..directories.row_count() {
                    if let Some(item) = directories.row_data(i) {
                        if item.path.as_str() == path_str {
                            already_exists = true;
                            break;
                        }
                    }
                }
                
                if already_exists {
                    println!("目录已存在: {}", path_str);
                } else {
                    let new_dir = DirectoryItem {
                        path: path_str.clone().into(),
                        selected: false,
                    };
                    
                    directories.push(new_dir);
                    // 强制更新UI
                    ui.set_directories(directories.clone().into());
                    println!("添加了新目录: {}", path_str);
                }
            }
            Ok(None) => {
                println!("用户取消了目录选择");
            }
            Err(e) => {
                println!("打开文件对话框时出错: {}", e);
            }
        }
    }
    
    /// 删除选中的目录
    fn remove_selected_directories(directories: &Rc<VecModel<DirectoryItem>>, ui: &AppWindow) {
        let mut removed_count = 0;
        let mut i = 0;
        
        while i < directories.row_count() {
            if let Some(item) = directories.row_data(i) {
                if item.selected {
                    directories.remove(i);
                    removed_count += 1;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        
        // 强制更新UI
        ui.set_directories(directories.clone().into());
        println!("删除了 {} 个目录", removed_count);
    }
    
    /// 切换目录的选择状态
    fn toggle_directory_selection(directories: &Rc<VecModel<DirectoryItem>>, ui: &AppWindow, index: i32) {
        if let Some(mut item) = directories.row_data(index as usize) {
            item.selected = !item.selected;
            directories.set_row_data(index as usize, item.clone());
            
            // 强制更新UI
            ui.set_directories(directories.clone().into());
            
            let status = if item.selected { "选中" } else { "取消选中" };
            println!("{}: {}", status, item.path);
        }
    }
      /// 运行UI
    pub fn run(&self) -> Result<(), slint::PlatformError> {
        // 确保初始搜索结果为空
        self.ui.set_search_results(slint::VecModel::from_slice(&[]));
        self.ui.run()
    }
    
    /// 获取当前所有目录
    pub fn get_directories(&self) -> Vec<DirectoryItem> {
        let mut dirs = Vec::new();
        for i in 0..self.directories.row_count() {
            if let Some(item) = self.directories.row_data(i) {
                dirs.push(item);
            }
        }
        dirs
    }
    
    /// 获取选中的目录
    pub fn get_selected_directories(&self) -> Vec<DirectoryItem> {
        self.get_directories()
            .into_iter()
            .filter(|dir| dir.selected)
            .collect()
    }
    
    /// 添加真实目录路径
    pub fn add_real_directory(&self, path: String) {
        let new_dir = DirectoryItem {
            path: path.into(),
            selected: false,
        };
        self.directories.push(new_dir);
    }
    
    /// 清空所有目录
    pub fn clear_directories(&self) {
        while self.directories.row_count() > 0 {
            self.directories.remove(0);
        }
    }
      /// 设置过滤器回调
    fn setup_filter_callbacks(&self) {
        let ui_weak = self.ui.as_weak();
        let current_filter = self.current_filter.clone();
        self.ui.on_filter_changed(move |filter_data| {
            if let Some(ui) = ui_weak.upgrade() {
                Self::handle_filter_changed(&ui, filter_data, &current_filter);
            }
        });
        
        // 初始化UI的过滤设置为当前保存的值
        self.sync_filter_to_ui();
    }    /// 处理过滤条件变化
    fn handle_filter_changed(ui: &AppWindow, filter_data: FilterData, current_filter: &Rc<std::cell::RefCell<SearchFilter>>) {
        println!("过滤条件已更改:");
        println!("  搜索隐藏文件: {}", filter_data.search_hidden_files);
        println!("  搜索隐藏文件夹: {}", filter_data.search_hidden_folders);
        println!("  搜索只读文件: {}", filter_data.search_readonly_files);
        println!("  文件大小范围: {} - {} MB", filter_data.min_file_size, filter_data.max_file_size);
        println!("  日期限制类型: {:?}", filter_data.date_limit_type);
        
        // 打印扩展的日期筛选信息
        if filter_data.date_limit_type == DateLimitType::Specific {
            println!("  起始日期: {}/{}/{}", filter_data.specific_year, filter_data.specific_month, filter_data.specific_day);
            println!("  结束日期: {}/{}/{}", filter_data.end_year, filter_data.end_month, filter_data.end_day);
        } else if filter_data.date_limit_type != DateLimitType::None {
            println!("  时间值: {} 单位: {} 内/外: {}", 
                filter_data.date_limit_value,
                match filter_data.time_unit {
                    0 => "天",
                    1 => "周", 
                    2 => "月",
                    3 => "年",
                    _ => "未知",
                },
                if filter_data.time_newer { "内" } else { "外" }
            );
        }
        
        println!("  正则表达式: '{}'", filter_data.regex_pattern);
        println!("  正则匹配目标: {:?}", filter_data.regex_target);
        println!("  记录哈希值: {}", filter_data.record_hash);
        
        // 转换枚举类型为整数
        let date_limit_type = match filter_data.date_limit_type {
            DateLimitType::None => 0,
            DateLimitType::Days => 1,
            DateLimitType::Weeks => 2,
            DateLimitType::Years => 3,
            DateLimitType::Months => 5,
            DateLimitType::Specific => 4,
        };
        
        let regex_target = match filter_data.regex_target {
            RegexTarget::FileName => 0,
            RegexTarget::FilePath => 1,
        };        // 直接调用修改后的SearchFilter::from_ui_data函数，传递所有参数
        let filter_result = SearchFilter::from_ui_data(
            filter_data.search_hidden_files,
            filter_data.search_hidden_folders,
            filter_data.search_readonly_files,
            filter_data.min_file_size,
            filter_data.max_file_size,
            date_limit_type,
            filter_data.date_limit_value,
            filter_data.specific_year,
            filter_data.specific_month,
            filter_data.specific_day,
            &filter_data.regex_pattern,
            regex_target,
            filter_data.record_hash,
            filter_data.time_newer,  // 传入是否"内"/"外"参数
            filter_data.end_year,    // 传入完整日期的结束年
            filter_data.end_month,   // 传入完整日期的结束月
            filter_data.end_day,     // 传入完整日期的结束日
        );
        
        match filter_result {
            Ok(filter) => {
                // 保存过滤器设置到UIHandler的状态中
                *current_filter.borrow_mut() = filter.clone();
                
                // 应用过滤器设置到SearchHelper
                SearchHelper::apply_filter_settings(&filter);
                
                // 更新UI中的保存的过滤器数据
                ui.set_saved_filter_data(filter_data);
                
                println!("✅ 过滤器创建成功并已保存: {:?}", filter);
            }
            Err(e) => {
                println!("❌ 创建过滤器失败: {}", e);
            }
        }
        println!("---");
    }
      /// 将当前保存的过滤器设置同步到UI
    fn sync_filter_to_ui(&self) {
        let filter_data = self.get_filter_data_for_ui();
        self.ui.set_saved_filter_data(filter_data);
        println!("已将保存的过滤器设置同步到UI");
    }
      /// 获取用于UI初始化的过滤器数据
    pub fn get_filter_data_for_ui(&self) -> FilterData {
        let filter = self.current_filter.borrow();        FilterData {
            search_hidden_files: filter.search_hidden_files,
            search_hidden_folders: filter.search_hidden_folders,
            search_readonly_files: filter.search_readonly_files,
            min_file_size: (filter.min_file_size / (1024 * 1024)) as i32, // 转换为MB
            max_file_size: (filter.max_file_size / (1024 * 1024)) as i32, // 转换为MB
            date_limit_type: match filter.date_limit {
                crate::filter::DateLimitType::None => DateLimitType::None,
                crate::filter::DateLimitType::Days(..) => DateLimitType::Days,
                crate::filter::DateLimitType::Weeks(..) => DateLimitType::Weeks,
                crate::filter::DateLimitType::Years(..) => DateLimitType::Years,
                crate::filter::DateLimitType::Months(..) => DateLimitType::Months,
                crate::filter::DateLimitType::Specific { .. } => DateLimitType::Specific,
            },
            date_limit_value: match filter.date_limit {
                crate::filter::DateLimitType::Days(v, _) => v,
                crate::filter::DateLimitType::Weeks(v, _) => v,
                crate::filter::DateLimitType::Years(v, _) => v,
                crate::filter::DateLimitType::Months(v, _) => v,
                _ => 1,
            },
            // 基本日期设置 - 使用minimum_*字段
            specific_year: match filter.date_limit {
                crate::filter::DateLimitType::Specific { minimum_year, .. } => minimum_year,
                _ => filter.start_year.unwrap_or(2024),
            },
            specific_month: match filter.date_limit {
                crate::filter::DateLimitType::Specific { minimum_month, .. } => minimum_month as i32,
                _ => filter.start_month.unwrap_or(1),
            },
            specific_day: match filter.date_limit {
                crate::filter::DateLimitType::Specific { minimum_day, .. } => minimum_day as i32,
                _ => filter.start_day.unwrap_or(1),
            },
            // specific_* 已经设置过了，这里只需要设置end_*
            end_year: 2025,
            end_month: 6,
            end_day: 22,
              // 快速选择设置 - 使用默认值
            time_unit: 0, // 默认为"天"
            time_newer: true, // 默认为"内"
            
            // 其他设置
            regex_pattern: filter.regex_pattern.as_ref().map(|r| r.as_str()).unwrap_or("").to_string().into(),
            regex_target: match filter.regex_target {
                crate::filter::RegexTarget::FileName => RegexTarget::FileName,
                crate::filter::RegexTarget::FilePath => RegexTarget::FilePath,
            },
            record_hash: filter.record_hash,
        }
    }
      /// 设置搜索相关的回调
    fn setup_search_callbacks(&self) {
        // 开始搜索按钮回调
        let ui_weak = self.ui.as_weak();
        let directories = self.directories.clone();
        let current_filter = self.current_filter.clone();
        let search_results = self.search_results.inner.clone();
          // 1. 搜索按钮回调
        let search_callback = move || {
            if let Some(_ui) = ui_weak.upgrade() {
                // 获取要搜索的目录
                let mut paths: Vec<PathBuf> = Vec::new();
                for i in 0..directories.row_count() {
                    if let Some(dir) = directories.row_data(i) {
                        if dir.selected {
                            paths.push(PathBuf::from(dir.path.as_str()));
                        }
                    }
                }
                
                if paths.is_empty() {
                    MessageDialog::new()
                        .set_type(MessageType::Info)
                        .set_title("提示")
                        .set_text("请选择至少一个目录进行搜索！")
                        .show_alert()
                        .unwrap();
                    return;
                }
                
                // 获取过滤条件
                let filter = current_filter.borrow().clone();
                
                // 检索文件
                let search_depth = 100; // 最大搜索深度
                let regex_str = match &filter.regex_pattern {
                    Some(pattern) => pattern.as_str().to_string(),
                    None => String::new(),
                };
                
                // 使用SearchHelper执行搜索
                let found_files = crate::helper::SearchHelper::perform_search(
                    &paths, 
                    search_depth, 
                    &regex_str,
                    &filter
                );
                println!("找到 {} 个文件", found_files.len());
                  // 清空之前的搜索结果
                while search_results.row_count() > 0 {
                    search_results.remove(0);
                }
                
                // 添加搜索结果到UI
                for file in &found_files {
                    let file_info = FileInfo {
                        path: file.path.to_string_lossy().to_string().into(),
                        name: file.name.clone().into(),
                        size: file.size as i32,
                        time: file.time as i32,
                        hash: file.hash.clone().into(),
                        selected: false, // 默认不选中
                    };
                    search_results.push(file_info);
                }
                  println!("搜索完成，找到 {} 个文件", search_results.row_count());
                
                // 先把结果转换为Vec后再传递给UI
                let mut file_infos = Vec::new();
                for i in 0..search_results.row_count() {
                    if let Some(info) = search_results.row_data(i) {
                        file_infos.push(info);
                    }
                }
                
                // 使用from_slice方法创建一个新的VecModel直接传递给UI
                _ui.set_search_results(slint::VecModel::from_slice(&file_infos));
            }
        };
        
        // 2. 导入结果按钮回调
        let ui_weak = self.ui.as_weak();
        let search_results = self.search_results.inner.clone();
        let import_callback = move || {
            if let Some(_ui) = ui_weak.upgrade() {                // 弹出文件选择对话框
                let dialog = FileDialog::new();
                let result = dialog.show_open_single_file();
                match result {
                    Ok(Some(file_path)) => {
                        // 读取JSON文件
                        match std::fs::read_to_string(&file_path) {
                            Ok(json_str) => {
                                // 解析JSON为Vec<SingleFileInformations>
                                match serde_json::from_str::<Vec<SingleFileInformations>>(&json_str) {                                    Ok(files) => {
                                        // 清空之前的搜索结果
                                        while search_results.row_count() > 0 {
                                            search_results.remove(0);
                                        }
                                          // 添加导入的结果
                                        for file in &files {
                                            let file_info = FileInfo {
                                                path: file.path.to_string_lossy().to_string().into(),
                                                name: file.name.clone().into(),
                                                size: file.size as i32,
                                                time: file.time as i32,
                                                hash: file.hash.clone().into(),
                                                selected: false, // 默认不选中
                                            };
                                            search_results.push(file_info);
                                        }
                                          // 先把结果转换为Vec后再传递给UI
                                        let mut file_infos = Vec::new();
                                        for i in 0..search_results.row_count() {
                                            if let Some(info) = search_results.row_data(i) {
                                                file_infos.push(info);
                                            }
                                        }
                                        
                                        // 使用from_slice方法创建一个新的VecModel直接传递给UI
                                        _ui.set_search_results(slint::VecModel::from_slice(&file_infos));
                                        
                                        println!("成功导入 {} 个搜索结果", files.len());
                                    },
                                    Err(e) => {
                                        MessageDialog::new()
                                            .set_type(MessageType::Error)
                                            .set_title("导入失败")
                                            .set_text(&format!("解析JSON文件失败: {}", e))
                                            .show_alert()
                                            .unwrap();
                                    }
                                }
                            },
                            Err(e) => {
                                MessageDialog::new()
                                    .set_type(MessageType::Error)
                                    .set_title("导入失败")
                                    .set_text(&format!("读取文件失败: {}", e))
                                    .show_alert()
                                    .unwrap();
                            }
                        }
                    },
                    Ok(None) => {
                        // 用户取消了选择
                    },
                    Err(e) => {
                        MessageDialog::new()
                            .set_type(MessageType::Error)
                            .set_title("错误")
                            .set_text(&format!("文件对话框错误: {}", e))
                            .show_alert()
                            .unwrap();
                    }
                }
            }
        };
        
        // 3. 设置目标文件夹按钮回调
        let ui_weak = self.ui.as_weak();
        let selected_paths = self.selected_paths.clone();
        let search_results = self.search_results.inner.clone();
        let select_folder_callback = move || {
            if let Some(_ui) = ui_weak.upgrade() {
                match FileDialog::new()
                    .show_open_single_dir() {
                    Ok(Some(folder_path)) => {
                        let path_str = folder_path.to_string_lossy().to_string();
                        
                        // 清空已有的路径并添加新的路径
                        while selected_paths.row_count() > 0 {
                            selected_paths.remove(0);
                        }
                        
                        // 添加新选择的路径
                        selected_paths.push(path_str.clone().into());
                        
                        // 打印当前选择的目标文件夹路径
                        println!("已设置目标文件夹: {}", path_str);
                        
                        // 打印当前选中的搜索结果文件列表
                        let mut selected_files = Vec::new();
                        for i in 0..search_results.row_count() {
                            if let Some(file_info) = search_results.row_data(i) {
                                if file_info.selected {
                                    selected_files.push(file_info.path.as_str().to_string());
                                }
                            }
                        }
                        println!("当前选中的文件: {:?}", selected_files);
                    },
                    Ok(None) => {
                        // 用户取消了选择
                    },
                    Err(e) => {
                        MessageDialog::new()
                            .set_type(MessageType::Error)
                            .set_title("错误")
                            .set_text(&format!("文件夹选择对话框错误: {}", e))
                            .show_alert()
                            .unwrap();
                    }
                }
            }
        };
        
        // 3.5 复制选中文件按钮回调
        let ui_weak = self.ui.as_weak();
        let search_results = self.search_results.inner.clone();
        let selected_paths = self.selected_paths.clone();
        let copy_files_callback = move || {
            if let Some(ui) = ui_weak.upgrade() {
                // 获取选中的文件
                let mut files = Vec::new();
                
                for i in 0..search_results.row_count() {
                    if let Some(file_info) = search_results.row_data(i) {
                        if file_info.selected {  // 只处理已选中的文件
                            files.push(SingleFileInformations {
                                path: PathBuf::from(file_info.path.as_str()),
                                name: file_info.name.as_str().to_string(),
                                size: file_info.size as u64,
                                time: file_info.time as u64,
                                hash: file_info.hash.as_str().to_string(),
                            });
                        }
                    }
                }
                
                // 检查是否有选中的文件
                if files.is_empty() {
                    MessageDialog::new()
                        .set_type(MessageType::Info)
                        .set_title("提示")
                        .set_text("请先选择要复制的文件！")
                        .show_alert()
                        .unwrap();
                    return;
                }
                
                // 检查是否有设置目标文件夹
                if selected_paths.row_count() == 0 {
                    MessageDialog::new()
                        .set_type(MessageType::Info)
                        .set_title("提示")
                        .set_text("请先设置一个目标文件夹！")
                        .show_alert()
                        .unwrap();
                    return;
                }
                
                // 确认复制
                let confirm = MessageDialog::new()
                    .set_type(MessageType::Warning)
                    .set_title("确认复制")
                    .set_text(&format!("确定要将选中的 {} 个文件复制到目标文件夹吗？", files.len()))
                    .show_confirm()
                    .unwrap_or(false);
                
                if confirm {
                    // 获取目标文件夹路径
                    if let Some(dest_path_str) = selected_paths.row_data(0) {
                        let dest_path = PathBuf::from(dest_path_str.as_str());
                        
                        // 使用SearchHelper复制文件
                        match SearchHelper::copy_files_to(&files, &dest_path) {
                            Ok(_) => {
                                MessageDialog::new()
                                    .set_type(MessageType::Info)
                                    .set_title("复制成功")
                                    .set_text(&format!("成功复制 {} 个文件到 {}", files.len(), dest_path.to_string_lossy()))
                                    .show_alert()
                                    .unwrap();
                            },
                            Err(e) => {
                                MessageDialog::new()
                                    .set_type(MessageType::Error)
                                    .set_title("复制失败")
                                    .set_text(&format!("复制文件时出错: {}", e))
                                    .show_alert()
                                    .unwrap();
                            }
                        }
                    }
                }
            }
        };
        
        // 3.6 移动选中文件按钮回调
        let ui_weak = self.ui.as_weak();
        let search_results = self.search_results.inner.clone();
        let selected_paths = self.selected_paths.clone();
        let move_files_callback = move || {
            if let Some(ui) = ui_weak.upgrade() {
                // 获取选中的文件
                let mut files = Vec::new();
                let mut indices_to_remove = Vec::new();
                
                for i in 0..search_results.row_count() {
                    if let Some(file_info) = search_results.row_data(i) {
                        if file_info.selected {  // 只处理已选中的文件
                            files.push(SingleFileInformations {
                                path: PathBuf::from(file_info.path.as_str()),
                                name: file_info.name.as_str().to_string(),
                                size: file_info.size as u64,
                                time: file_info.time as u64,
                                hash: file_info.hash.as_str().to_string(),
                            });
                            indices_to_remove.push(i);
                        }
                    }
                }
                
                // 检查是否有选中的文件
                if files.is_empty() {
                    MessageDialog::new()
                        .set_type(MessageType::Info)
                        .set_title("提示")
                        .set_text("请先选择要移动的文件！")
                        .show_alert()
                        .unwrap();
                    return;
                }
                
                // 检查是否有设置目标文件夹
                if selected_paths.row_count() == 0 {
                    MessageDialog::new()
                        .set_type(MessageType::Info)
                        .set_title("提示")
                        .set_text("请先设置一个目标文件夹！")
                        .show_alert()
                        .unwrap();
                    return;
                }
                
                // 确认移动
                let confirm = MessageDialog::new()
                    .set_type(MessageType::Warning)
                    .set_title("确认移动")
                    .set_text(&format!("确定要将选中的 {} 个文件移动到目标文件夹吗？此操作将改变文件位置！", files.len()))
                    .show_confirm()
                    .unwrap_or(false);
                
                if confirm {
                    // 获取目标文件夹路径
                    if let Some(dest_path_str) = selected_paths.row_data(0) {
                        let dest_path = PathBuf::from(dest_path_str.as_str());
                        
                        // 使用SearchHelper移动文件
                        match SearchHelper::move_files_to(&files, &dest_path) {
                            Ok(_) => {
                                MessageDialog::new()
                                    .set_type(MessageType::Info)
                                    .set_title("移动成功")
                                    .set_text(&format!("成功移动 {} 个文件到 {}", files.len(), dest_path.to_string_lossy()))
                                    .show_alert()
                                    .unwrap();
                                
                                // 从搜索结果中移除已移动的文件
                                // 从后向前移除，避免索引问题
                                indices_to_remove.sort_by(|a, b| b.cmp(a));
                                for index in indices_to_remove {
                                    search_results.remove(index);
                                }
                                
                                // 更新UI显示
                                let mut file_infos = Vec::new();
                                for i in 0..search_results.row_count() {
                                    if let Some(info) = search_results.row_data(i) {
                                        file_infos.push(info);
                                    }
                                }
                                ui.set_search_results(slint::VecModel::from_slice(&file_infos));
                            },
                            Err(e) => {
                                MessageDialog::new()
                                    .set_type(MessageType::Error)
                                    .set_title("移动失败")
                                    .set_text(&format!("移动文件时出错: {}", e))
                                    .show_alert()
                                    .unwrap();
                            }
                        }
                    }
                }
            }
        };
        
        // 4. 删除选中文件按钮回调
        let ui_weak = self.ui.as_weak();
        let search_results = self.search_results.inner.clone();
        let delete_files_callback = move || {
            if let Some(ui) = ui_weak.upgrade() {
                // 获取选中的文件
                let mut files = Vec::new();
                let mut indices_to_remove = Vec::new();
                
                for i in 0..search_results.row_count() {
                    if let Some(file_info) = search_results.row_data(i) {
                        if file_info.selected {  // 只处理已选中的文件
                            files.push(SingleFileInformations {
                                path: PathBuf::from(file_info.path.as_str()),
                                name: file_info.name.as_str().to_string(),
                                size: file_info.size as u64,
                                time: file_info.time as u64,
                                hash: file_info.hash.as_str().to_string(),
                            });
                            indices_to_remove.push(i);
                        }
                    }
                }
                
                // 检查是否有选中的文件
                if files.is_empty() {
                    MessageDialog::new()
                        .set_type(MessageType::Info)
                        .set_title("提示")
                        .set_text("请先选择要删除的文件！")
                        .show_alert()
                        .unwrap();
                    return;
                }
                
                // 确认删除
                let confirm = MessageDialog::new()
                    .set_type(MessageType::Warning)
                    .set_title("确认删除")
                    .set_text(&format!("确定要删除选中的 {} 个文件吗？此操作不可恢复！", files.len()))
                    .show_confirm()
                    .unwrap_or(false);
                
                if confirm {
                    // 使用SearchHelper删除文件
                    match SearchHelper::delete_files(&files) {
                        Ok(_) => {
                            MessageDialog::new()
                                .set_type(MessageType::Info)
                                .set_title("删除成功")
                                .set_text(&format!("成功删除 {} 个文件", files.len()))
                                .show_alert()
                                .unwrap();
                                
                            // 从搜索结果中移除已删除的文件
                            // 从后向前移除，避免索引问题
                            indices_to_remove.sort_by(|a, b| b.cmp(a));
                            for index in indices_to_remove {
                                search_results.remove(index);
                            }
                            
                            // 更新UI显示
                            let mut file_infos = Vec::new();
                            for i in 0..search_results.row_count() {
                                if let Some(info) = search_results.row_data(i) {
                                    file_infos.push(info);
                                }
                            }
                            ui.set_search_results(slint::VecModel::from_slice(&file_infos));
                        },
                        Err(e) => {
                            MessageDialog::new()
                                .set_type(MessageType::Error)
                                .set_title("删除失败")
                                .set_text(&format!("删除文件时出错: {}", e))
                                .show_alert()
                                .unwrap();
                        }
                    }
                }
            }
        };
        
        // 5. 映射文件按钮回调
        let ui_weak = self.ui.as_weak();
        let search_results = self.search_results.inner.clone();
        let selected_paths = self.selected_paths.clone();
        let map_files_callback = move || {
            if let Some(_ui) = ui_weak.upgrade() {
                if search_results.row_count() == 0 {
                    MessageDialog::new()
                        .set_type(MessageType::Info)
                        .set_title("提示")
                        .set_text("没有搜索结果可以进行映射！")
                        .show_alert()
                        .unwrap();
                    return;
                }
                
                // if selected_paths.row_count() == 0 {
                //     MessageDialog::new()
                //         .set_type(MessageType::Info)
                //         .set_title("提示")
                //         .set_text("请先设置一个目标文件夹！")
                //         .show_alert()
                //         .unwrap();
                //     return;
                // }
                
//                 MessageDialog::new()
//                     .set_type(MessageType::Info)
//                     .set_title("提示")
//                     .set_text("接下来你将选择一个文件夹作为存放映射的文件夹\n
// 映射操作将会把当前选中的搜索结果在目标文件夹内的文件维持原有的文件结构复制到这个文件夹中\n
// 即以该文件夹为根重新建立从目标文件夹到所有选中文件的文件结构")
//                     .show_alert()
//                     .unwrap();
                MessageDialog::new()
                    .set_type(MessageType::Info)
                    .set_title("提示")
                    .set_text("接下来你将分别选择两个文件夹作为映射的起始和目标文件夹\n
映射操作将会把当前选中的搜索结果在起始文件夹内的文件维持原有的文件结构复制到目标文件夹中\n
即以目标文件夹为根重新建立从起始文件夹到所有选中文件的文件结构")
                    .show_alert()
                    .unwrap();
                
                // 弹出选择目标文件夹的对话框
                let result = FileDialog::new().show_open_single_dir();
                match result {
                    Ok(Some(source_path)) => {
                        // 获取目标文件夹路径

                        let result2 = FileDialog::new().show_open_single_dir();
                        match result2 {
                            Ok(Some(dest_path)) => {
                                // 获取搜索结果
                                let mut files = Vec::new();
                                for i in 0..search_results.row_count() {
                                    if let Some(file_info) = search_results.row_data(i) {
                                        if file_info.selected {
                                            // 只处理已选中的文件
                                            files.push(SingleFileInformations {
                                                path: PathBuf::from(file_info.path.as_str()),
                                                name: file_info.name.as_str().to_string(),
                                                size: file_info.size as u64,
                                                time: file_info.time as u64,
                                                hash: file_info.hash.as_str().to_string(),
                                            });
                                        }
                                    }
                                }

                                // 使用SearchHelper执行映射
                                SearchHelper::map_files(&files, &source_path, &dest_path);

                                MessageDialog::new()
                                    .set_type(MessageType::Info)
                                    .set_title("映射完成")
                                    .set_text(&format!("成功映射 {} 个文件到 {}", files.len(), dest_path.to_string_lossy()))
                                    .show_alert()
                                    .unwrap();
                            },
                            Ok(None) => {
                                // 用户取消了选择
                            },
                            Err(e) => {
                                MessageDialog::new()
                                    .set_type(MessageType::Error)
                                    .set_title("错误")
                                    .set_text(&format!("目标文件夹选择对话框错误: {}", e))
                                    .show_alert()
                                    .unwrap();
                            }
                        }
                    },
                    Ok(None) => {
                        // 用户取消了选择
                    },
                    Err(e) => {
                        MessageDialog::new()
                            .set_type(MessageType::Error)
                            .set_title("错误")
                            .set_text(&format!("起始文件夹选择对话框错误: {}", e))
                            .show_alert()
                            .unwrap();
                    }
                }
            }
        };
        
        // 6. 去重展示按钮回调
        let ui_weak = self.ui.as_weak();
        let search_results = self.search_results.inner.clone();
        let remove_duplicates_callback = move || {
            if let Some(updated_ui) = ui_weak.upgrade() {
                // 获取搜索结果
            let mut files = Vec::new();
            for i in 0..search_results.row_count() {
                if let Some(file_info) = search_results.row_data(i) {
                    files.push(SingleFileInformations {
                        path: PathBuf::from(file_info.path.as_str()),
                        name: file_info.name.as_str().to_string(),
                        size: file_info.size as u64,
                        time: file_info.time as u64,
                        hash: file_info.hash.as_str().to_string(),
                    });
                }
            }
            
            if files.is_empty() {
                MessageDialog::new()
                    .set_type(MessageType::Info)
                    .set_title("提示")
                    .set_text("没有搜索结果可以进行去重！")
                    .show_alert()
                    .unwrap();
                return;
            }
            
            if files[0].hash.is_empty() {
                MessageDialog::new()
                    .set_type(MessageType::Info)
                    .set_title("提示")
                    .set_text("搜索结果中没有哈希值，请在过滤设置中启用'记录哈希值'选项！")
                    .show_alert()
                    .unwrap();
                return;
            }
            
            // MessageDialog::new()
            //     .set_type(MessageType::Info)
            //     .set_title("提示")
            //     .set_text("正在进行去重处理，请稍候...")
            //     .show_alert()
            //     .unwrap();

            // 去重处理 - 使用SearchHelper
            let original_count = files.len();
            crate::search_file::unique_files(&mut files);
            let unique_count = files.len();

            // 打印去重结果
            println!("去重前文件数量: {}", original_count);
            println!("去重后文件数量: {}", unique_count);
            for file in &files {
                println!("保留文件: {} - SHA256:{}", file.name, file.hash);
            }
            
            if original_count == unique_count {
                MessageDialog::new()
                    .set_type(MessageType::Info)
                    .set_title("去重结果")
                    .set_text("没有重复文件，去重操作未做任何改变。")
                    .show_alert()
                    .unwrap();
                return;
            }

            // // 更新搜索结果
            
            while search_results.row_count() > 0 {
                search_results.remove(search_results.row_count() as usize - 1);
            }

            println!("清空之前的搜索结果");

            // while search_results.row_count() > 0 {
            //     search_results.remove(0);
            // }
            let mut file_infos = Vec::new();
            for file in &files {
                let file_info = FileInfo {
                    path: file.path.to_string_lossy().to_string().into(),
                    name: file.name.clone().into(),
                    size: file.size as i32,
                    time: file.time as i32,
                    hash: file.hash.clone().into(),
                    selected: false, // 默认不选中
                };
                search_results.push(file_info.clone());
                file_infos.push(file_info);
            }

            // 显示转化为Vec<FileInfo>后的结果
            println!("转换为Vec后，搜索结果数量: {}", file_infos.len());
            
            // 使用已有的UI引用设置搜索结果
            updated_ui.set_search_results(slint::VecModel::from_slice(&file_infos));
            
            MessageDialog::new()
                .set_type(MessageType::Info)
                .set_title("去重完成")
                .set_text(&format!("原有 {} 个文件，去重后剩余 {} 个文件", original_count, unique_count))
                .show_alert()
                .unwrap();
            }
        };
        
        // 7. 排序按钮回调

        
        // 8. 打开文件夹回调
        let open_folder_callback = move |path: SharedString| {
            let path_str = path.as_str();
            let path_obj = PathBuf::from(path_str);
            
            if let Some(parent) = path_obj.parent() {
                #[cfg(target_os = "windows")]
                {
                    std::process::Command::new("explorer")
                        .arg(parent)
                        .spawn()
                        .expect("Failed to open explorer");
                }
                
                #[cfg(target_os = "linux")]
                {
                    std::process::Command::new("xdg-open")
                        .arg(parent)
                        .spawn()
                        .expect("Failed to open file browser");
                }
                
                #[cfg(target_os = "macos")]
                {
                    std::process::Command::new("open")
                        .arg(parent)
                        .spawn()
                        .expect("Failed to open finder");
                }
            }
        };        
        // 新增导出结果按钮回调
        let ui_weak = self.ui.as_weak();
        let search_results = self.search_results.inner.clone();
        let export_callback = move || {
            if let Some(_ui) = ui_weak.upgrade() {
                // 检查是否有搜索结果
                if search_results.row_count() == 0 {
                    MessageDialog::new()
                        .set_type(MessageType::Info)
                        .set_title("提示")
                        .set_text("没有搜索结果可以导出！")
                        .show_alert()
                        .unwrap();
                    return;
                }
                
                // 弹出文件保存对话框
                let dialog = FileDialog::new()
                    .set_filename("search_results.json");
                
                if let Ok(Some(file_path)) = dialog.show_save_single_file() {
                    // 获取搜索结果
                    let mut files = Vec::new();
                    for i in 0..search_results.row_count() {
                        if let Some(file_info) = search_results.row_data(i) {
                            files.push(SingleFileInformations {
                                path: PathBuf::from(file_info.path.as_str()),
                                name: file_info.name.as_str().to_string(),
                                size: file_info.size as u64,
                                time: file_info.time as u64,
                                hash: file_info.hash.as_str().to_string(),
                            });
                        }
                    }
                    
                    // 使用SearchHelper导出结果
                    match SearchHelper::export_results(&files, Some(file_path.to_str().unwrap())) {
                        Ok(_) => {
                            MessageDialog::new()
                                .set_type(MessageType::Info)
                                .set_title("导出成功")
                                .set_text(&format!("成功导出 {} 个搜索结果", files.len()))
                                .show_alert()
                                .unwrap();
                        },
                        Err(e) => {
                            MessageDialog::new()
                                .set_type(MessageType::Error)
                                .set_title("导出失败")
                                .set_text(&format!("导出结果时出错: {}", e))
                                .show_alert()
                                .unwrap();
                        }
                    }
                }
            }
        };
        
        // 新增树状显示按钮回调
        let ui_weak = self.ui.as_weak();
        let search_results = self.search_results.inner.clone();
        let show_tree_view_callback = move || {
            if let Some(_ui) = ui_weak.upgrade() {
                // 检查是否有搜索结果
                if search_results.row_count() == 0 {
                    MessageDialog::new()
                        .set_type(MessageType::Info)
                        .set_title("提示")
                        .set_text("没有搜索结果可以显示！")
                        .show_alert()
                        .unwrap();
                    return;
                }
                
                // 获取搜索结果
                let mut files = Vec::new();
                for i in 0..search_results.row_count() {
                    if let Some(file_info) = search_results.row_data(i) {
                        files.push(SingleFileInformations {
                            path: PathBuf::from(file_info.path.as_str()),
                            name: file_info.name.as_str().to_string(),
                            size: file_info.size as u64,
                            time: file_info.time as u64,
                            hash: file_info.hash.as_str().to_string(),
                        });
                    }
                }
                
                // 调用get_tree函数获取树状结构
                let tree_content = crate::search_file::build_tree::get_tree(&files);
                
                // 保存到search_result.txt文件
                let dialog = FileDialog::new()
                    .set_filename("search_result.txt");
                if let Ok(Some(file_path)) = dialog.show_save_single_file() {
                    match std::fs::write(file_path, tree_content) {
                        Ok(_) => {
                            MessageDialog::new()
                                .set_type(MessageType::Info)
                                .set_title("树状结构生成成功")
                                .set_text(&format!("成功生成树状结构!"))
                                .show_alert()
                                .unwrap();
                        },
                        Err(e) => {
                            MessageDialog::new()
                                .set_type(MessageType::Error)
                                .set_title("生成失败")
                                .set_text(&format!("保存树状结构时出错: {}", e))
                                .show_alert()
                                .unwrap();
                        }
                    }
                }
            }
        };
          
        // 8. 文件项选择状态变化回调
        let ui_weak = self.ui.as_weak();
        let search_results = self.search_results.inner.clone();
        let item_selected_changed = move |index: i32, selected: bool| {
            if let Some(ui) = ui_weak.upgrade() {
                // 更新指定索引的文件选择状态
                if let Some(mut file_info) = search_results.row_data(index as usize) {
                    file_info.selected = selected;
                    search_results.set_row_data(index as usize, file_info);
                    
                    // 计算已选择的数量并打印
                    let mut selected_count = 0;
                    for i in 0..search_results.row_count() {
                        if let Some(info) = search_results.row_data(i) {
                            if info.selected {
                                selected_count += 1;
                            }
                        }
                    }
                    println!("已选择 {}/{} 个文件", selected_count, search_results.row_count());
                    
                    // 打印当前选中的文件列表
                    let mut selected_files = Vec::new();
                    for i in 0..search_results.row_count() {
                        if let Some(file_info) = search_results.row_data(i) {
                            if file_info.selected {
                                selected_files.push(file_info.path.as_str().to_string());
                            }
                        }
                    }
                    println!("当前选中的文件: {:?}", selected_files);
                    
                    // 更新UI中的选中计数
                    ui.set_selected_count(selected_count);
                    
                    // 转换为Vec后再更新UI
                    let mut file_infos = Vec::new();
                    for i in 0..search_results.row_count() {
                        if let Some(info) = search_results.row_data(i) {
                            file_infos.push(info);
                        }
                    }
                    ui.set_search_results(slint::VecModel::from_slice(&file_infos));
                }
            }
        };
        
        // 9. 全选/取消全选回调
        let ui_weak = self.ui.as_weak();
        let search_results = self.search_results.inner.clone();
        let select_all = move |selected: bool| {
            if let Some(ui) = ui_weak.upgrade() {
                // 更新所有文件的选择状态
                let mut file_infos = Vec::new();
                for i in 0..search_results.row_count() {
                    if let Some(mut info) = search_results.row_data(i) {
                        info.selected = selected;
                        search_results.set_row_data(i, info.clone());
                        file_infos.push(info);
                    }
                }
                
                // 更新UI
                ui.set_search_results(slint::VecModel::from_slice(&file_infos));
                
                println!("{}全选 {} 个文件", 
                    if selected { "" } else { "取消" }, 
                    search_results.row_count()
                );
                
                // 如果是全选，打印所有文件路径
                if selected {
                    let mut selected_files = Vec::new();
                    for i in 0..search_results.row_count() {
                        if let Some(file_info) = search_results.row_data(i) {
                            selected_files.push(file_info.path.as_str().to_string());
                        }
                    }
                    println!("当前选中的文件: {:?}", selected_files);
                    ui.set_selected_count(selected_files.len() as i32);
                } else {
                    println!("当前选中的文件: []");
                    ui.set_selected_count(0);
                }
            }
        };

        // 7. 排序功能回调
        let ui_weak = self.ui.as_weak();
        let search_results = self.search_results.inner.clone();
        let sort_results_callback = move |sort_type: SharedString, reversed: bool| {
            if let Some(ui) = ui_weak.upgrade() {
                // 获取搜索结果
                let mut files = Vec::new();
                for i in 0..search_results.row_count() {
                    if let Some(file_info) = search_results.row_data(i) {
                        files.push(SingleFileInformations {
                            path: PathBuf::from(file_info.path.as_str()),
                            name: file_info.name.as_str().to_string(),
                            size: file_info.size as u64,
                            time: file_info.time as u64,
                            hash: file_info.hash.as_str().to_string(),
                        });
                    }
                }
                
                if files.is_empty() {
                    MessageDialog::new()
                        .set_type(MessageType::Info)
                        .set_title("提示")
                        .set_text("没有搜索结果可以进行排序！")
                        .show_alert()
                        .unwrap();
                    return;
                }
                
                // 获取排序类型和方向
                let sort_type_str = sort_type.as_str();
                println!("开始排序，排序方式: {}, 方向: {}", sort_type_str, if reversed {"从大到小"} else {"从小到大"});
                
                // 执行排序操作
                SearchHelper::sort_results(&mut files, sort_type_str, reversed);
                
                // 清空现有的搜索结果
                while search_results.row_count() > 0 {
                    search_results.remove(search_results.row_count() as usize - 1);
                }
                
                println!("清空之前的搜索结果，准备更新排序后的结果");
                
                // 创建新的搜索结果
                let mut file_infos = Vec::new();
                for file in &files {
                    let file_info = FileInfo {
                        path: file.path.to_string_lossy().to_string().into(),
                        name: file.name.clone().into(),
                        size: file.size as i32,
                        time: file.time as i32,
                        hash: file.hash.clone().into(),
                        selected: false, // 默认不选中
                    };
                    search_results.push(file_info.clone());
                    file_infos.push(file_info);
                }
                
                println!("转换为Vec后，搜索结果数量: {}", file_infos.len());
                
                // 更新UI
                ui.set_search_results(slint::VecModel::from_slice(&file_infos));
                
                MessageDialog::new()
                    .set_type(MessageType::Info)
                    .set_title("排序完成")
                    .set_text(&format!("已按{}{}排序 {} 个文件", 
                        match sort_type_str {
                            "name" => "文件名",
                            "size" => "文件大小",
                            "time" => "修改日期",
                            "path" => "文件路径",
                            _ => "未知方式",
                        },
                        if reversed {
                            "从大到小"
                        } else {
                            "从小到大"
                        },
                        files.len()
                    ))
                    .show_alert()
                    .unwrap();
            }
        };

        // 通过全局接口暴露回调
        self.ui.on_handle_search_clicked(search_callback);
        self.ui.on_handle_import_results(import_callback);
        self.ui.on_handle_export_results(export_callback);
        self.ui.on_handle_show_tree_view(show_tree_view_callback);
        self.ui.on_handle_select_folder(select_folder_callback);
        self.ui.on_handle_move_selected_files(move_files_callback);
        self.ui.on_handle_copy_selected_files(copy_files_callback);
        self.ui.on_handle_delete_selected_files(delete_files_callback);
        self.ui.on_handle_map_files(map_files_callback);
        self.ui.on_handle_remove_duplicates(remove_duplicates_callback);
        self.ui.on_handle_open_folder(open_folder_callback);
        self.ui.on_handle_sort_results(sort_results_callback);
        self.ui.on_item_selected_changed(item_selected_changed);
        self.ui.on_select_all(select_all);
          // 设置初始数据绑定
        // 先将search_results模型转换为Vec，然后再创建一个新的VecModel
        let mut file_infos = Vec::new();
        for i in 0..self.search_results.inner.row_count() {
            if let Some(info) = self.search_results.inner.row_data(i) {
                file_infos.push(info);            }
        }
          self.ui.set_search_results(slint::VecModel::from_slice(&file_infos));
        self.ui.set_selected_paths(self.selected_paths.clone().into());
    }
}
