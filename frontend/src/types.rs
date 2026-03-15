use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id:           String,
    pub name:         String,
    pub group:        String,
    #[serde(rename = "type")]
    pub product_type: String,
    pub price:        f64,
    pub image_url:    String,
    pub in_stock:     bool,
    pub stock_count:  i32,
    pub attributes:   Option<HashMap<String, serde_json::Value>>,
    pub created_at:   String,
    pub updated_at:   String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductListItem {
    pub id:           String,
    pub name:         String,
    pub group:        String,
    #[serde(rename = "type")]
    pub product_type: String,
    pub price:        f64,
    pub image_url:    String,
    pub in_stock:     bool,
    pub stock_count:  i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductListResponse {
    pub data:  Vec<ProductListItem>,
    pub total: i32,
    pub page:  i32,
    pub limit: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductGroup {
    pub group: String,
    pub label: String,
    pub types: Vec<ProductTypeEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductTypeEntry {
    #[serde(rename = "type")]
    pub type_key: String,
    pub label:    String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id:         String,
    pub email:      String,
    pub name:       String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user:  User,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email:    String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email:    String,
    pub name:     String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id:   String,
    pub product_name: String,
    pub product_type: String,
    pub quantity:     i32,
    pub unit_price:   f64,
    pub line_total:   f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShippingAddress {
    pub facility_name:  String,
    pub site_code:      String,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city:           String,
    pub country:        String,
    pub postal_code:    String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub id:               String,
    pub user_id:          String,
    pub status:           String,
    pub items:            Vec<OrderItem>,
    pub shipping_address: ShippingAddress,
    pub subtotal:         f64,
    pub shipping_cost:    f64,
    pub total:            f64,
    pub notes:            Option<String>,
    pub created_at:       String,
    pub updated_at:       String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderListResponse {
    pub data:  Vec<Order>,
    pub total: i32,
    pub page:  i32,
    pub limit: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub items:            Vec<CreateOrderItem>,
    pub shipping_address: ShippingAddress,
    pub notes:            Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateOrderItem {
    pub product_id: String,
    pub quantity:   i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CartItem {
    pub product:  ProductListItem,
    pub quantity: i32,
}

impl CartItem {
    pub fn line_total(&self) -> f64 {
        self.product.price * self.quantity as f64
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role:    String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatRequest {
    pub query:      String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatResponse {
    pub answer:  String,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefundResponse {
    pub valid:          bool,
    pub order_id:       Option<String>,
    pub reason:         String,
    pub extracted_data: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProductFilters {
    pub group:  Option<String>,
    pub type_:  Option<String>,
    pub search: Option<String>,
    pub page:   i32,
    pub limit:  i32,
}

impl ProductFilters {
    pub fn new() -> Self {
        Self { page: 1, limit: 20, ..Default::default() }
    }

    pub fn to_query_string(&self) -> String {
        let mut params = vec![];
        if let Some(g) = &self.group  { params.push(format!("group={}", g)); }
        if let Some(t) = &self.type_  { params.push(format!("type={}", t)); }
        if let Some(s) = &self.search { params.push(format!("search={}", s)); }
        params.push(format!("page={}", self.page));
        params.push(format!("limit={}", self.limit));
        params.join("&")
    }
}