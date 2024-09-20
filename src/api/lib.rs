use crate::api::dawn::*;
use crate::config::account::EmailAccount;
use crate::utils::email_util::*;
use crate::utils::errors::CustomError;
use crate::utils::errors::CustomError::{DawnAPIError, EmailAPIError};
use crate::utils::parse_image::*;
use crate::utils::xpath_util::*;
use chrono::Utc;
use reqwest::header;
use reqwest::StatusCode;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

pub fn get_dawn_date_format() -> String {
    // 获取当前时间，并以 ISO 8601 格式输出毫秒精度
    let current_time = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    // 将 "+00:00" 替换为 "Z"
    let current_time = current_time.replace("+00:00", "Z");
    current_time
}

impl New for DawnAPI {
    fn new(account: EmailAccount) -> DawnAPI {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));
        headers.insert(
            header::ACCEPT_LANGUAGE,
            header::HeaderValue::from_static("en-US,en;q=0.9"),
        );
        headers.insert(
            header::ORIGIN,
            header::HeaderValue::from_static("chrome-extension://fpdkjdnhkakefebpekbdhillbhonfjjp"),
        );
        headers.insert(
            header::HeaderName::from_static("priority"),
            header::HeaderValue::from_static("u=1, i"),
        );
        headers.insert(header::USER_AGENT, header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"));
        let client = reqwest::Client::builder()
            .tls_built_in_webpki_certs(false)
            .danger_accept_invalid_certs(true) // 禁用 SSL 证书验证
            .build()
            .unwrap();
        DawnAPI {
            client: client,
            account: account,
            headers: headers,
            token: "".to_string(),
        }
    }
}

impl InvokeApi for DawnAPI {
    async fn get_puzzle_id(&self) -> Result<String, Box<dyn Error + Send>> {
        let resp = self
            .client
            .get("https://www.aeropres.in/chromeapi/dawn/v1/puzzle/get-puzzle")
            .headers(self.headers.clone())
            .send()
            .await
            .map_err(|e| CustomError::DawnAPIError(e.to_string()))
            .unwrap();
        if resp.status() != StatusCode::OK && resp.status() != StatusCode::CREATED {
            return Err(Box::new(DawnAPIError(
                "Dawn API Error: Status Code is not 200".to_string(),
            )));
        }
        let json: Value = resp
            .json()
            .await
            .map_err(|e| CustomError::DawnAPIError(e.to_string()))
            .unwrap();
        if json["success"] == true {
            let puzzle_id = json["puzzle_id"].as_str().unwrap().to_string();
            println!("puzzle_id: {}", puzzle_id);
            Ok(puzzle_id)
        } else {
            println!("{:}", json["message"]);
            Err(Box::new(DawnAPIError(
                json["message"]
                    .as_str()
                    .unwrap_or("Unknown Error")
                    .to_string(),
            )))
        }
    }

    async fn get_puzzle_data(&self, puzzle_id: String) -> Result<String, Box<dyn Error + Send>> {
        let mut query: HashMap<String, String> = HashMap::new();
        query.insert("puzzle_id".to_string(), puzzle_id.clone());
        let resp = self
            .client
            .get("https://www.aeropres.in/chromeapi/dawn/v1/puzzle/get-puzzle-image")
            .query(&query)
            .headers(self.headers.clone())
            .send()
            .await
            .map_err(|e| CustomError::DawnAPIError(e.to_string()))
            .unwrap();
        let json: Value = resp
            .json()
            .await
            .map_err(|e| CustomError::DawnAPIError(e.to_string()))
            .unwrap();
        if json["success"] == true {
            Ok(json["imgBase64"].as_str().unwrap().to_string())
        } else {
            println!("{:}", json["message"]);
            Err(Box::new(DawnAPIError(
                json["message"]
                    .as_str()
                    .unwrap_or("Unknown Error")
                    .to_string(),
            )))
        }
    }

    async fn send_login(
        &mut self,
        puzzle_id: String,
        puzzle_ans: String,
    ) -> Result<bool, Box<dyn Error + Send>> {
        let current_time = get_dawn_date_format();
        let data = LoginData {
            username: self.account.email.clone(),
            password: self.account.password.clone(),
            logindata: Logindata {
                _v: "1.0.7".to_string(),
                datetime: current_time,
            },
            puzzle_id: puzzle_id,
            ans: puzzle_ans,
        };

        let login_data = json!(data);

        let resp = self
            .client
            .post("https://www.aeropres.in//chromeapi/dawn/v1/user/login/v2")
            .json(&login_data)
            .headers(self.headers.clone())
            .send()
            .await
            .map_err(|e| CustomError::DawnAPIError(e.to_string()))
            .unwrap()
            .json::<Value>()
            .await
            .map_err(|e| CustomError::DawnAPIError(e.to_string()))
            .unwrap();
        if resp.get("status").and_then(Value::as_bool).unwrap_or(false) == true {
            self.token = resp["data"]["token"].as_str().unwrap().to_string();
            self.headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(format!("Bearer {}", self.token).as_str())
                    .map_err(|e| CustomError::DawnAPIError(e.to_string()))
                    .unwrap(),
            );
            println!("login_token: {}", self.token);
            Ok(true)
        } else {
            println!("{:?}", resp);
            Err(Box::new(DawnAPIError(
                resp.get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Unknown Error")
                    .to_string(),
            )))
        }
    }

    async fn get_and_solve_captcha(&self) -> Result<Captcha, Box<dyn Error + Send>> {
        let puzzle_id = self.get_puzzle_id().await;
        if let Err(e) = puzzle_id {
            return Err(e);
        }
        let puzzle_id = puzzle_id.unwrap();
        let puzzle_data_req = self.get_puzzle_data(puzzle_id.clone()).await;
        if let Err(e) = puzzle_data_req {
            return Err(e);
        }
        let puzzle_data = puzzle_data_req.unwrap();
        let puzzle_ans = get_ocr_result_from_b64(puzzle_data);
        if let Ok(puzzle_ans) = puzzle_ans {
            if puzzle_ans.len() == 6 {
                return Ok(Captcha {
                    puzzle_id,
                    puzzle_ans,
                });
            } else {
                return Err(Box::new(CustomError::CaptchaError(
                    "验证码识别非6位!".to_string(),
                )));
            }
        }

        println!("puzzle_ans: {:?}", puzzle_ans);
        Err(Box::new(CustomError::CaptchaError(
            "验证码识别出错！".to_string(),
        )))
    }

    async fn get_point(&self) -> Result<String, Box<dyn Error + Send>> {
        let resp = self
            .client
            .get("https://www.aeropres.in/api/atom/v1/userreferral/getpoint")
            .headers(self.headers.clone())
            .send()
            .await
            .map_err(|e| CustomError::DawnAPIError(e.to_string()))
            .unwrap()
            .json::<Value>()
            .await
            .map_err(|e| CustomError::DawnAPIError(e.to_string()))
            .unwrap();
        if resp.get("status").and_then(Value::as_bool).unwrap_or(false) == true {
            println!("Get Point Success!");
            let points = resp["data"]["rewardPoint"]["points"].as_f64().unwrap();
            let twitter_x_id_points = resp["data"]["rewardPoint"]["twitter_x_id_points"]
                .as_f64()
                .unwrap();
            let discordid_points = resp["data"]["rewardPoint"]["discordid_points"]
                .as_f64()
                .unwrap();
            let telegramid_points = resp["data"]["rewardPoint"]["telegramid_points"]
                .as_f64()
                .unwrap();
            let total_points = points + twitter_x_id_points + discordid_points + telegramid_points;
            println!("total_point: {}", total_points);
            Ok(total_points.to_string())
        } else {
            println!("{:?}", resp);
            Err(Box::new(DawnAPIError(
                resp.get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Unknown Error")
                    .to_string(),
            )))
        }
    }

    async fn heart_beat(&self) -> Result<(), Box<dyn Error + Send>> {
        let data = json!({
            "username": self.account.email.clone(),
            "extensionid": "fpdkjdnhkakefebpekbdhillbhonfjjp",
            "numberoftabs": 0,
            "_v": "1.0.7",
        });
        let resp = self
            .client
            .post("https://www.aeropres.in/chromeapi/dawn/v1/userreward/keepalive")
            .json(&data)
            .headers(self.headers.clone())
            .send()
            .await
            .map_err(|e| CustomError::DawnAPIError(e.to_string()))
            .unwrap();
        let response_data = resp.json::<Value>().await;
        match response_data {
            Ok(data) => {
                println!("data:{}", data);
                let result = data
                    .get("success")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if result {
                    return Ok(());
                } else {
                    return Err(Box::new(DawnAPIError("Heartbeat failed!".to_string())));
                }
            }
            Err(err) => {
                return Err(Box::new(DawnAPIError(
                    format!("Heartbeat failed:{}", err).to_string(),
                )));
            }
        }
    }

    async fn send_regist(
        &self,
        puzzle_id: String,
        puzzle_ans: String,
        referral_code: String,
    ) -> Result<(), Box<dyn Error + Send>> {
        let mut r_code = "".to_string();
        if referral_code == "".to_string() {
            r_code = "kxgm14b7".to_string();
        }
        let register_data = json!({
            "firstname": self.account.email.split('@').next().unwrap_or(""),
            "lastname": self.account.email.split('@').next().unwrap_or(""),
            "email": self.account.email,
            "mobile": "",
            "password": self.account.password,
            "country": "+91".to_string(),
            "referralCode": r_code,
            "puzzle_id": puzzle_id,
            "ans": puzzle_ans,
        });
        let resp = self
            .client
            .post("https://www.aeropres.in/chromeapi/dawn/v1/puzzle/validate-register")
            .headers(self.headers.clone())
            .json(&register_data)
            .send()
            .await
            .map_err(|e| CustomError::DawnAPIError(e.to_string()))
            .unwrap();
        let response_data = resp.json::<Value>().await;
        match response_data {
            Ok(data) => {
                println!("data:{}", data);
                let result = data
                    .get("success")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if result {
                    return Ok(());
                } else {
                    if let Some(message) = data.get("message").and_then(Value::as_str) {
                        // 检查消息是否包含 "email already exists"
                        if message.contains("email already exists") {
                            return Ok(());
                        } else if message.contains("Incorrect answer") {
                            return Err(Box::new(DawnAPIError("验证码识别错误,重试".to_string())));
                        } else {
                            return Err(Box::new(DawnAPIError(message.to_string())));
                        }
                    }
                    return Err(Box::new(DawnAPIError("Register failed!".to_string())));
                }
            }
            Err(err) => {
                return Err(Box::new(DawnAPIError(
                    format!("Register failed:{}", err).to_string(),
                )));
            }
        }
    }
}

impl Jobs for DawnAPI {
    async fn work_flow(&mut self) -> Result<(), Box<dyn Error + Send>> {
        let login_result = self.ensure_login().await;
        match login_result {
            Ok(result) => {
                if result {
                    println!("Login success!");
                    self.keep_heartbeat().await;
                } else {
                    println!("Login failed!");
                }
            }
            Err(err) => {
                println!("Login failed: {}", err);
            }
        }
        Ok(())
    }

    async fn heartbeat_once(&mut self, token: String) -> Result<(), Box<dyn Error + Send>> {
        self.headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(format!("Bearer {}", token).as_str())
                .map_err(|e| CustomError::DawnAPIError(e.to_string()))
                .unwrap(),
        );
        match self.heart_beat().await {
            Ok(_) => {
                println!("{} Heartbeat success!", self.account.email);
                if let Ok(total_point) = self.get_point().await {
                    println!("{}total_point: {}", self.account.email, total_point);
                } else {
                    println!("获取积分失败");
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                return Ok(());
            }
            Err(err) => {
                println!("{}Heartbeat failed: {}", self.account.email, err);
                return Err(err);
            }
        }
    }

    async fn keep_heartbeat(&mut self) {
        loop {
            match self.heart_beat().await {
                Ok(_) => {
                    println!("Heartbeat success!");
                    if let Ok(total_point) = self.get_point().await {
                        println!("total_point: {}", total_point);
                    } else {
                        println!("获取积分失败");
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                }
                Err(err) => {
                    println!("Heartbeat failed: {}", err);
                }
            }
        }
    }

    async fn login(&mut self) -> Result<bool, Box<dyn Error + Send>> {
        let solved_captcha = self.get_and_solve_captcha().await;
        if let Err(e) = solved_captcha {
            return Err(e);
        }
        let solved_captcha = solved_captcha.unwrap();
        self.send_login(solved_captcha.puzzle_id, solved_captcha.puzzle_ans)
            .await
    }

    async fn ensure_login(&mut self) -> Result<bool, Box<dyn Error + Send>> {
        loop {
            let login_result = self.login().await;
            match login_result {
                Ok(result) => {
                    if result == true {
                        println!("登录成功");
                        return Ok(true);
                    }
                }
                Err(err) => {
                    let message = err.to_string();
                    if message.contains("Email not verified") {
                        println!("邮箱未验证,失败");
                        return Err(err);
                    } else {
                        println!("登录失败: {}", err);
                    }
                }
            }
        }
    }

    async fn ensure_regist(&mut self) -> Result<(), Box<dyn Error + Send>> {
        loop {
            let regist_result = self.regist_once().await;
            match regist_result {
                Ok(_) => {
                    return Ok(());
                }
                Err(err) => {
                    let message = err.to_string();
                    if message.contains("验证码识别错误") {
                        println!("验证码识别错误,重试");
                    } else {
                        println!("注册失败: {}", err);
                        return Err(err);
                    }
                }
            }
        }
    }

    async fn regist_once(&mut self) -> Result<(), Box<dyn Error + Send>> {
        // let solved_ = self.get_and_solve_captcha().await;
        // if let Err(err) = solved_ {
        //     return Err(err);
        // }
        // let solved_captcha = solved_.unwrap();
        // // todo 设置邀请码
        // let send = self
        //     .send_regist(
        //         solved_captcha.puzzle_id,
        //         solved_captcha.puzzle_ans,
        //         "".to_string(),
        //     )
        //     .await;
        // if let Err(err) = send {
        //     return Err(err);
        // }
        // 等待10秒,收邮件
        // tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let email_content = fetch_dawn_email_link(
            &self.account.email,
            &self.account.password,
            &self.account.imap,
        )
        .await;
        if let Err(err) = email_content {
            return Err(err);
        }
        let content = email_content.unwrap();
        let regist_element = find_and_return_regist_url(content, ".mail1 a").await;
        if regist_element.is_err() {
            return Err(Box::new(EmailAPIError("解析邮件失败".to_string())));
        }
        let regist_url = regist_element.unwrap();

        let res = self
            .client
            .get(regist_url)
            .headers(self.headers.clone())
            .send()
            .await;
        if res.is_err() {
            return Err(Box::new(DawnAPIError("打开激活链接失败".to_string())));
        }
        let response_test = res.unwrap().text().await;

        if response_test.is_err() {
            return Err(Box::new(DawnAPIError("激活失败".to_string())));
        }

        if let Ok(message) = response_test {
            if message.contains("email verified successfully") {
                println!("{}激活成功", self.account.email);
                return Ok(());
            } else if message.contains("already verified your email address") {
                println!("{}已激活", self.account.email);
                return Ok(());
            } else {
                return Err(Box::new(DawnAPIError(format!("激活失败:{}", message))));
            }
        }

        return Err(Box::new(DawnAPIError("激活失败".to_string())));
    }
    // async fn social_task(&mut self) {}
}
