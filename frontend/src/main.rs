use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod context;
mod hooks;
mod pages;
mod route;
mod services;
mod types;

use context::auth::AuthProvider;
use context::cart::CartProvider;
use route::Route;

fn switch(route: Route) -> Html {
    match route {
        Route::Landing                  => html! { <pages::landing::Landing /> },
        Route::Catalog                  => html! { <pages::catalog::Catalog /> },
        Route::ProductDetail { id }     => html! { <pages::product_detail::ProductDetail {id} /> },
        Route::Compare { id }           => html! { <pages::compare::Compare {id} /> },
        Route::Cart                     => html! { <pages::cart::Cart /> },
        Route::Checkout                 => html! { <pages::checkout::Checkout /> },
        Route::OrderConfirmation { id } => html! { <pages::order_confirmation::OrderConfirmation {id} /> },
        Route::Orders                   => html! { <pages::orders::Orders /> },
        Route::OrderDetail { id }       => html! { <pages::order_detail::OrderDetail {id} /> },
        Route::Refund { order_id }      => html! { <pages::refund::Refund {order_id} /> },
        Route::Login                    => html! { <pages::login::Login /> },
        Route::Register                 => html! { <pages::register::Register /> },
        Route::Profile                  => html! { <pages::profile::Profile /> },
        Route::NotFound                 => html! { <pages::not_found::NotFound /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <AuthProvider>
                <CartProvider>
                    <div class="min-h-screen bg-navy">
                        <components::layout::navbar::Navbar />
                        <main>
                            <Switch<Route> render={switch} />
                        </main>
                    </div>
                </CartProvider>
            </AuthProvider>
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}