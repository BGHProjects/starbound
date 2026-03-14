use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod pages;
mod services;
mod types;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/catalog")]
    Catalog,
    #[at("/product/:id")]
    Product { id: u32 },
    #[at("/checkout")]
    Checkout,
    #[at("/account")]
    Account,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home     => html! { <pages::home::Home /> },
        Route::Catalog  => html! { <pages::catalog::Catalog /> },
        Route::Checkout => html! { <pages::checkout::Checkout /> },
        Route::Account  => html! { <pages::account::Account /> },
        _               => html! {
            <div class="p-8 text-orange font-orbitron">
                { "404 — Page not found" }
            </div>
        },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <div class="min-h-screen bg-navy">
                <components::navbar::Navbar />
                <main>
                    <Switch<Route> render={switch} />
                </main>
            </div>
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}