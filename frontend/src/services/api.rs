use gloo_net::http::Request;
use serde::{de::DeserializeOwned, Serialize};

const BASE_URL: &str = "http://localhost:8000/api";

pub struct ApiClient;

impl ApiClient {
    pub async fn get<T: DeserializeOwned>(
        path:  &str,
        token: Option<&str>,
    ) -> Result<T, String> {
        let url = format!("{}{}", BASE_URL, path);
        let mut req = Request::get(&url);
        if let Some(t) = token {
            req = req.header("Authorization", &format!("Bearer {}", t));
        }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if resp.ok() {
            resp.json::<T>().await.map_err(|e| e.to_string())
        } else {
            Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()))
        }
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(
        path:  &str,
        body:  &B,
        token: Option<&str>,
    ) -> Result<T, String> {
        let url = format!("{}{}", BASE_URL, path);
        let mut req = Request::post(&url)
            .header("Content-Type", "application/json");
        if let Some(t) = token {
            req = req.header("Authorization", &format!("Bearer {}", t));
        }
        let resp = req
            .body(serde_json::to_string(body).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.ok() {
            resp.json::<T>().await.map_err(|e| e.to_string())
        } else {
            Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()))
        }
    }

    pub async fn put<T: DeserializeOwned>(
        path:  &str,
        token: Option<&str>,
    ) -> Result<T, String> {
        let url = format!("{}{}", BASE_URL, path);
        let mut req = Request::put(&url);
        if let Some(t) = token {
            req = req.header("Authorization", &format!("Bearer {}", t));
        }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if resp.ok() {
            resp.json::<T>().await.map_err(|e| e.to_string())
        } else {
            Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()))
        }
    }
}