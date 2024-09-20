use crate::api::dawn::*;
use crate::config::account::*;
use crate::tokio::time::Duration;
use crate::utils::errors::CustomError;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use tokio;
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    imap_settings: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbAccount {
    account: EmailAccount,
    token: String,
}

pub async fn get_settings_from_file() -> Result<Settings, Box<dyn Error + Send>> {
    // // 读取 YAML 文件内容
    let content = fs::read_to_string("./settings.yaml").await;
    if let Ok(setting_content) = content {
        // 反序列化 YAML 内容到 Rust 结构体
        let settings: Result<Settings, serde_yaml::Error> = serde_yaml::from_str(&setting_content);
        if let Ok(settings_obj) = settings {
            return Ok(settings_obj);
        } else {
            return Err(Box::new(CustomError::EmailAPIError(
                "Invalid YAML content".to_string(),
            )));
        }
    }
    Err(Box::new(CustomError::EmailAPIError(
        "Invalid YAML content".to_string(),
    )))
}

pub async fn get_account_from_line(line: String, settings: Settings) -> EmailAccount {
    // 使用分隔符拆分字符串
    let parts: Vec<&str> = line.split("----").collect();
    let email = parts[0];
    let password = parts[1];
    let email_parts: Vec<&str> = email.split("@").collect();
    let host = email_parts[1];
    let imap_host = settings.imap_settings.get(host).unwrap();
    EmailAccount::load_account(email, password, imap_host)
}

pub async fn do_work_flow(
    account: EmailAccount,
    token: String,
) -> Result<(), Box<dyn Error + Send>> {
    let mut client = DawnAPI::new(account);
    if token != "".to_string() {
        client.token = token;
    }
    client.work_flow().await.expect("工作流失败");
    Ok(())
}

pub async fn do_regist(account: EmailAccount) -> Result<(), Box<dyn Error + Send>> {
    let mut client = DawnAPI::new(account);
    let result = client.ensure_regist().await;
    if result.is_ok() {
        println!("{}注册成功", client.account.email);
    } else {
        println!("注册失败");
    }
    Ok(())
}

// async fn login_and_heartbeat(client){

// }

pub async fn work_via_redis(
    db_ua: String,
    db_pw: String,
    db_host: String,
    db_port: String,
    db_num: String,
) -> Result<(), Box<dyn Error + Send>> {
    // 指定 host, port, username, password 的 Redis 连接字符串
    let redis_url = format!(
        "redis://{}:{}@{}:{}/{}",
        db_ua, db_pw, db_host, db_port, db_num
    );
    let client = redis::Client::open(redis_url)
        .map_err(|e| CustomError::CustomRedisError(e.to_string()))
        .unwrap();
    let mut con = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| CustomError::CustomRedisError(e.to_string()))
        .unwrap();

    let settings = get_settings_from_file().await.unwrap();

    loop {
        // 打开文件
        let file = File::open("./src/emails/gmail.txt")
            .await
            .map_err(|e| CustomError::EmailFileError(e.to_string()))
            .unwrap();
        let reader = BufReader::new(file);
        // 逐行读取文件
        let mut lines = reader.lines();
        while let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| CustomError::EmailFileError(e.to_string()))
            .unwrap()
        {
            let account = get_account_from_line(line.to_string(), settings.clone()).await;
            println!("正在处理: {}", line);
            let db_info: Result<String, CustomError> = con
                .get(account.email.clone())
                .await
                .map_err(|e| CustomError::RedisInfoError(e.to_string()));
            match db_info {
                Ok(db_info) => {
                    if let Ok(mut db_account) = serde_json::from_str::<DbAccount>(&db_info)
                        .map_err(|e| CustomError::RedisInfoError(e.to_string()))
                    {
                        let mut client = DawnAPI::new(db_account.account.clone());
                        if db_account.token == "".to_string() {
                            println!("Token不存在，登陆后保活，写token");
                            let login_res = client.ensure_login().await;
                            match login_res {
                                Ok(_) => {
                                    client.heartbeat_once(client.token.clone()).await?;
                                    // 写token
                                    db_account.token = client.token.clone();
                                    let db_str = serde_json::to_string(&db_account)
                                        .map_err(|e| CustomError::RedisInfoError(e.to_string()))
                                        .unwrap();
                                    println!("db写入token:{}", db_str);
                                    con.set(db_account.account.email.clone(), db_str)
                                        .await
                                        .map_err(|e| CustomError::RedisInfoError(e.to_string()))
                                        .unwrap()
                                }
                                Err(e) => {
                                    println!(
                                        "{}登录失败:{}",
                                        db_account.account.email,
                                        e.to_string()
                                    );
                                    continue;
                                }
                            }
                        } else {
                            println!("Token存在，直接保活");
                            client.heartbeat_once(db_account.token).await?;
                            // todo 处理token失效的问题
                        }
                    } else {
                        // todo 改为删除键值，重新登录
                        return Err(Box::new(CustomError::RedisInfoError(
                            "Invalid JSON".to_string(),
                        )));
                    }
                    continue;
                }
                Err(_) => {
                    println!("账户不存在，登陆后保活，写token");
                    let mut client = DawnAPI::new(account.clone());
                    let login_res = client.ensure_login().await;
                    match login_res {
                        Ok(_) => {
                            client.heartbeat_once(client.token.clone()).await?;
                            let db_account = DbAccount {
                                account: account.clone(),
                                token: client.token.clone(),
                            };
                            // 写token
                            let db_str = serde_json::to_string(&db_account)
                                .map_err(|e| CustomError::RedisInfoError(e.to_string()))
                                .unwrap();
                            println!("db写入token:{}", db_str);
                            con.set(account.email.clone(), db_str)
                                .await
                                .map_err(|e| CustomError::RedisInfoError(e.to_string()))
                                .unwrap()
                        }
                        Err(e) => {
                            println!("{}登录失败:{}", account.email, e.to_string());
                            continue;
                        }
                    }
                }
            };
            // 异步等待三秒
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }
    Ok(())
}
