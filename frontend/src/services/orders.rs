use crate::services::api::ApiClient;
use crate::types::{Order, OrderListResponse, CreateOrderRequest};

pub struct OrderService;

impl OrderService {
    pub async fn list(token: &str, page: i32) -> Result<OrderListResponse, String> {
        ApiClient::get(&format!("/orders?page={}", page), Some(token)).await
    }

    pub async fn get(id: &str, token: &str) -> Result<Order, String> {
        ApiClient::get(&format!("/orders/{}", id), Some(token)).await
    }

    pub async fn create(req: &CreateOrderRequest, token: &str) -> Result<Order, String> {
        ApiClient::post("/orders", req, Some(token)).await
    }

    pub async fn cancel(id: &str, token: &str) -> Result<Order, String> {
        ApiClient::put(&format!("/orders/{}/cancel", id), Some(token)).await
    }
}