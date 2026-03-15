use crate::services::api::ApiClient;
use crate::types::{AuthResponse, LoginRequest, RegisterRequest, User};

pub struct AuthService;

impl AuthService {
    pub async fn login(req: LoginRequest) -> Result<AuthResponse, String> {
        ApiClient::post("/auth/login", &req, None).await
    }

    pub async fn register(req: RegisterRequest) -> Result<AuthResponse, String> {
        ApiClient::post("/auth/register", &req, None).await
    }

    pub async fn me(token: &str) -> Result<User, String> {
        ApiClient::get("/auth/me", Some(token)).await
    }
}