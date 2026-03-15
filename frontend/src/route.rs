use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Landing,
    #[at("/catalog/:group")]
    CatalogFiltered { group: String },
    #[at("/catalog")]
    Catalog,
    #[at("/product/:id")]
    ProductDetail { id: String },
    #[at("/product/:id/compare")]
    Compare { id: String },
    #[at("/cart")]
    Cart,
    #[at("/checkout")]
    Checkout,
    #[at("/order-confirmation/:id")]
    OrderConfirmation { id: String },
    #[at("/orders")]
    Orders,
    #[at("/orders/:id")]
    OrderDetail { id: String },
    #[at("/refund/:order_id")]
    Refund { order_id: String },
    #[at("/login")]
    Login,
    #[at("/register")]
    Register,
    #[at("/profile")]
    Profile,
    #[not_found]
    #[at("/404")]
    NotFound,
}