// 隐藏Windows控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ui_handler;
mod filter;
mod search_file;


use std::error::Error;
use ui_handler::UIHandler;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    // 创建UI处理器
    let ui_handler = UIHandler::new()?;
    
    // 设置所有回调
    ui_handler.setup_callbacks();
    
    // 运行应用程序
    ui_handler.run()?;

    Ok(())
}