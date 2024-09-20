use crate::config::account::*;
use serde::Serialize;
use std::error::Error;

#[derive(Serialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
    pub logindata: Logindata,
    pub puzzle_id: String,
    pub ans: String,
}

#[derive(Serialize)]
pub struct Logindata {
    pub _v: String,
    pub datetime: String,
}

#[derive(Serialize)]
pub struct Captcha {
    pub puzzle_id: String,
    pub puzzle_ans: String,
}

pub struct DawnAPI {
    pub client: reqwest::Client,
    pub account: EmailAccount,
    pub headers: reqwest::header::HeaderMap,
    pub token: String,
}

pub trait InvokeApi {
    async fn get_and_solve_captcha(&self) -> Result<Captcha, Box<dyn Error + Send>>;
    async fn get_puzzle_id(&self) -> Result<String, Box<dyn Error + Send>>;
    async fn get_puzzle_data(&self, puzzle_id: String) -> Result<String, Box<dyn Error + Send>>;
    async fn send_login(
        &mut self,
        puzzle_id: String,
        puzzle_ans: String,
    ) -> Result<bool, Box<dyn Error + Send>>;
    async fn get_point(&self) -> Result<String, Box<dyn Error + Send>>;
    async fn heart_beat(&self) -> Result<(), Box<dyn Error + Send>>;
    async fn send_regist(
        &self,
        puzzle_id: String,
        puzzle_ans: String,
        referral_code: String,
    ) -> Result<(), Box<dyn Error + Send>>;
}

pub trait New {
    fn new(account: EmailAccount) -> DawnAPI;
}

pub trait Jobs {
    async fn login(&mut self) -> Result<bool, Box<dyn Error + Send>>;
    async fn regist_once(&mut self) -> Result<(), Box<dyn Error + Send>>;
    // async fn social_task(&mut self);
    async fn keep_heartbeat(&mut self);
    async fn ensure_login(&mut self) -> Result<bool, Box<dyn Error + Send>>;
    async fn ensure_regist(&mut self) -> Result<(), Box<dyn Error + Send>>;
    async fn work_flow(&mut self) -> Result<(), Box<dyn Error + Send>>;
    async fn heartbeat_once(&mut self, token: String) -> Result<(), Box<dyn Error + Send>>;
}
