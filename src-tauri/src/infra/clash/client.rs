use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::{json, Value};

use crate::core::domain::error::{AppError, AppResult};
use crate::core::domain::types::{ClashStatus, GlobalSettings};

pub struct ClashClient {
    client: Client,
    api_url: String,
    secret: String,
}

impl ClashClient {
    pub fn from_global(global: &GlobalSettings) -> AppResult<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .map_err(|error| AppError::Network(format!("初始化 HTTP client 失败: {error}")))?;

        Ok(Self {
            client,
            api_url: format!("http://127.0.0.1:{}/configs", global.clash_port),
            secret: global.clash_secret.clone(),
        })
    }

    pub fn get_status(&self) -> AppResult<ClashStatus> {
        let response = self
            .client
            .get(&self.api_url)
            .headers(self.headers()?)
            .send()
            .map_err(|error| AppError::Network(format!("读取 Clash 状态失败: {error}")))?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "读取 Clash 状态失败，HTTP {}",
                response.status()
            )));
        }

        let body = response
            .json::<Value>()
            .map_err(|error| AppError::Network(format!("解析 Clash 返回失败: {error}")))?;

        let tun = body.get("tun").cloned();
        let system_proxy = body
            .get("system-proxy")
            .or_else(|| body.get("system_proxy"))
            .or_else(|| body.get("systemProxy"))
            .and_then(extract_bool_like);

        Ok(ClashStatus { tun, system_proxy })
    }

    pub fn set_proxy(
        &self,
        tun: bool,
        system_proxy: Option<bool>,
        retries: usize,
    ) -> AppResult<()> {
        let mut payloads = Vec::new();

        if let Some(system_proxy_value) = system_proxy {
            payloads.push(json!({ "tun": { "enable": tun }, "system-proxy": system_proxy_value }));
            payloads.push(json!({ "tun": tun, "system-proxy": system_proxy_value }));
            payloads.push(json!({ "tun": { "enable": tun }, "system_proxy": system_proxy_value }));
            payloads.push(json!({ "tun": tun, "system_proxy": system_proxy_value }));
            payloads.push(json!({ "system-proxy": system_proxy_value }));
            payloads.push(json!({ "system_proxy": system_proxy_value }));
            payloads.push(json!({ "tun": { "enable": tun }, "systemProxy": system_proxy_value }));
        }

        payloads.push(json!({ "tun": { "enable": tun } }));
        payloads.push(json!({ "tun": tun }));

        let mut last_error = String::new();

        for _ in 0..retries.max(1) {
            for payload in &payloads {
                match self
                    .client
                    .patch(&self.api_url)
                    .headers(self.headers()?)
                    .json(payload)
                    .send()
                {
                    Ok(response) if response.status().is_success() => {
                        std::thread::sleep(std::time::Duration::from_millis(200));
                        let current = self.get_status()?;
                        let tun_ok = extract_tun_enabled(current.tun.as_ref()) == tun;
                        let system_ok = match system_proxy {
                            Some(expected) => current
                                .system_proxy
                                .map(|value| value == expected)
                                .unwrap_or(true),
                            None => true,
                        };
                        if tun_ok && system_ok {
                            return Ok(());
                        }
                        last_error = format!("写入后校验不一致: {payload}");
                    }
                    Ok(response) => {
                        last_error = format!("HTTP {}", response.status());
                    }
                    Err(error) => {
                        last_error = error.to_string();
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(400));
        }

        Err(AppError::Network(format!(
            "下发 Clash 代理状态失败: {last_error}"
        )))
    }
}

fn extract_bool_like(value: &Value) -> Option<bool> {
    if let Some(boolean) = value.as_bool() {
        return Some(boolean);
    }

    if let Some(number) = value.as_i64() {
        return Some(number != 0);
    }

    if let Some(text) = value.as_str() {
        let normalized = text.trim().to_ascii_lowercase();
        if normalized == "true"
            || normalized == "1"
            || normalized == "on"
            || normalized == "enabled"
        {
            return Some(true);
        }
        if normalized == "false"
            || normalized == "0"
            || normalized == "off"
            || normalized == "disabled"
        {
            return Some(false);
        }
    }

    if let Some(object) = value.as_object() {
        if let Some(inner) = object.get("enable").and_then(extract_bool_like) {
            return Some(inner);
        }
        if let Some(inner) = object.get("enabled").and_then(extract_bool_like) {
            return Some(inner);
        }
        if let Some(inner) = object.get("value").and_then(extract_bool_like) {
            return Some(inner);
        }
    }

    None
}

pub fn extract_tun_enabled(value: Option<&Value>) -> bool {
    let Some(v) = value else {
        return false;
    };

    if let Some(enabled) = v.as_bool() {
        return enabled;
    }

    if let Some(enabled) = v.get("enable").and_then(Value::as_bool) {
        return enabled;
    }

    false
}

impl ClashClient {
    fn headers(&self) -> AppResult<HeaderMap> {
        let mut headers = HeaderMap::new();
        if !self.secret.trim().is_empty() {
            let value =
                HeaderValue::from_str(&format!("Bearer {}", self.secret)).map_err(|error| {
                    AppError::Network(format!("Authorization Header 无效: {error}"))
                })?;
            headers.insert(AUTHORIZATION, value);
        }
        Ok(headers)
    }
}
