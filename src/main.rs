#[path = "api/mod.rs"]
mod api;

#[path = "config/mod.rs"]
mod config;

#[path = "utils/mod.rs"]
mod utils;

#[path = "worker/mod.rs"]
mod worker;

use crate::worker::lib::*;
use serde_yaml;
use std::error::Error;
use thiserror::Error;
use tokio;
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Error, Debug)]
pub enum MainThreadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Custom error: {0}")]
    Custom(String),
}

// redis循环work
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send>> {
    work_via_redis(
        "".to_string(),
        "".to_string(),
        "127.0.0.1".to_string(),
        "6379".to_string(),
        "2".to_string(),
    )
    .await
}

// 循环注册
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     // 打开文件
//     let file = File::open("./src/emails/outlook.txt").await?;
//     let reader = BufReader::new(file);

//     // 逐行读取文件
//     let mut lines = reader.lines();
//     while let Some(line) = lines.next_line().await? {
//         let line = line.clone();
//         println!("Processing line: {}", line);
//         // // 读取 YAML 文件内容
//         let content = fs::read_to_string("./settings.yaml").await;
//         if let Ok(setting_content) = content {
//             // 反序列化 YAML 内容到 Rust 结构体
//             let settings: Result<Settings, serde_yaml::Error> =
//                 serde_yaml::from_str(&setting_content);
//             if let Ok(settings_obj) = settings {
//                 let account = get_account_from_line(line, settings_obj).await;
//                 do_regist(account).await.expect("msg");
//             }
//         }
//     }
//     Ok(())
// }

// 全流程异步
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     // 打开文件
//     let file = File::open("./src/emails/outlook.txt").await?;
//     let reader = BufReader::new(file);

//     // 逐行读取文件
//     let mut lines = reader.lines();

//     // 用来存储任务的句柄
//     let mut handles: Vec<tokio::task::JoinHandle<Result<_, Box<MainThreadError>>>> = vec![];

//     while let Some(line) = lines.next_line().await? {
//         // 为每一行创建一个异步任务
//         let line = line.clone();
//         // let thread_settings: Arc<Mutex<Settings>> = Arc::clone(&settings);
//         let handle: tokio::task::JoinHandle<Result<_, Box<MainThreadError>>> =
//             tokio::spawn(async move {
//                 println!("Processing line: {}", line);
//                 // // 读取 YAML 文件内容
//                 let content = fs::read_to_string("./settings.yaml").await;
//                 if let Ok(setting_content) = content {
//                     // 反序列化 YAML 内容到 Rust 结构体
//                     let settings: Result<Settings, serde_yaml::Error> =
//                         serde_yaml::from_str(&setting_content);
//                     if let Ok(settings_obj) = settings {
//                         // let setting_map: HashMap<String, String> = settings_obj.imap_settings;
//                         // // 使用分隔符拆分字符串
//                         // let parts: Vec<&str> = line.split("----").collect();
//                         // let email = parts[0];
//                         // let password = parts[1];
//                         // let email_parts: Vec<&str> = email.split("@").collect();
//                         // let host = email_parts[1];
//                         // let imap_host = setting_map.get(host).unwrap();
//                         // let account = EmailAccount::load_account(email, password, imap_host);
//                         // println!("account: {:?}", account);
//                         let account = get_account_from_line(line, settings_obj).await;
//                         do_regist(account).await.expect("msg");
//                     }
//                 }
//                 Err::<String, Box<MainThreadError>>(Box::new(MainThreadError::Custom(
//                     "error".to_string(),
//                 )))
//                 // Ok("ok".to_string())
//             });
//         handles.push(handle);
//     }

//     // 等待所有任务完成
//     for handle in handles {
//         let handle_result = handle.await;
//         if let Ok(result) = handle_result {
//             println!("Result: {:?}", result)
//         }
//     }
//     Ok(())
// }
