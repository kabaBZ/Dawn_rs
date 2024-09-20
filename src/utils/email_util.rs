// use imap::Session;
use crate::utils::errors::CustomError;
use crate::utils::errors::CustomError::EmailAPIError;
use mailparse::{parse_mail, MailHeaderMap};
use native_tls::TlsConnector;
use std::{collections::HashSet, error::Error};

pub async fn fetch_dawn_email_link(
    username: &str,
    password: &str,
    imap: &str,
) -> Result<String, Box<dyn Error + Send>> {
    // 创建一个 TLS 连接
    let tls = TlsConnector::builder()
        .build()
        .map_err(|e| CustomError::EmailAPIError(e.to_string()))
        .unwrap();

    // 连接到 IMAP 服务器（例如 Gmail）
    let client = imap::connect((imap, 993), imap, &tls);
    if let Err(_) = client {
        return Err(Box::new(EmailAPIError(format!("链接邮件服务器失败"))));
    }
    let client = client.unwrap();

    // 登录
    let session = client.login(username, password);
    if let Err(_) = session {
        return Err(Box::new(EmailAPIError(format!("邮箱登录失败"))));
    }

    let mut session = session.unwrap();
    // 选择 "INBOX" 邮箱文件夹
    session
        .select("INBOX")
        .map_err(|e| CustomError::EmailAPIError(e.to_string()))
        .unwrap();

    // 获取邮件的 UID 列表
    let inbox_uids: HashSet<u32> = session
        .search("ALL")
        .map_err(|e| CustomError::EmailAPIError(e.to_string()))
        .unwrap();

    // 选择 "Junk" 邮箱文件夹
    session
        .select("Junk")
        .map_err(|e| CustomError::EmailAPIError(e.to_string()))
        .unwrap();

    // 获取邮件的 UID 列表
    let junk_uids: HashSet<u32> = session
        .search("ALL")
        .map_err(|e| CustomError::EmailAPIError(e.to_string()))
        .unwrap();

    let uids: HashSet<u32> = inbox_uids.union(&junk_uids).cloned().collect();

    // 遍历并获取每封邮件的内容
    for uid in uids.iter() {
        let messages = session
            .fetch(uid.to_string(), "RFC822")
            .map_err(|e| CustomError::EmailAPIError(e.to_string()))
            .unwrap();

        for message in messages.iter() {
            if let Some(body) = message.body() {
                // 使用 mailparse 解析邮件
                let parsed = parse_mail(body)
                    .map_err(|e| CustomError::EmailAPIError(e.to_string()))
                    .unwrap();

                // 获取发件人地址
                if let Some(from_header) = parsed.headers.get_first_header("From") {
                    if from_header.get_value() != "hello@dawninternet.com".to_string() {
                        // 获取邮件正文
                        continue;
                    }
                }

                let html_content = parsed.parts().last().unwrap().get_body().unwrap();
                // println!(
                //     "HTML content:\n{}",
                //     html_content,
                //     // String::from_utf8(html_content.unwrap()).unwrap()
                // );
                // 登出并关闭连接
                session
                    .logout()
                    .map_err(|e| CustomError::EmailAPIError(e.to_string()))
                    .unwrap();
                return Ok(html_content);
            }
        }
    }
    Err(Box::new(EmailAPIError("No Dawn email found.".to_string())))
}
