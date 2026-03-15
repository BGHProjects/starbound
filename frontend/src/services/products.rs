use crate::services::api::ApiClient;
use crate::types::{Product, ProductListResponse, ProductFilters};

pub struct ProductService;

impl ProductService {
    pub async fn list(filters: &ProductFilters) -> Result<ProductListResponse, String> {
        ApiClient::get(&format!("/products?{}", filters.to_query_string()), None).await
    }

    pub async fn get(id: &str) -> Result<Product, String> {
        ApiClient::get(&format!("/products/{}", id), None).await
    }

    pub async fn get_similar(
        product_type: &str,
        exclude_id:   &str,
    ) -> Result<ProductListResponse, String> {
        let mut result: ProductListResponse =
            ApiClient::get(&format!("/products?type={}&limit=3", product_type), None).await?;
        result.data.retain(|p| p.id != exclude_id);
        result.data.truncate(2);
        Ok(result)
    }
}