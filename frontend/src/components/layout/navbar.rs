use yew::prelude::*;
use yew_router::prelude::*;
use crate::route::Route;
use crate::context::auth::AuthContext;
use crate::context::cart::CartContext;

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let cart = use_context::<CartContext>().expect("CartContext not found");
    let item_count = cart.item_count();

    html! {
        <nav class="bg-navy2 border-b border-border h-16 px-6 flex items-center justify-between sticky top-0 z-50">
            <Link<Route> to={Route::Landing}>
                <div class="flex items-center gap-3 cursor-pointer">
                    <div class="w-8 h-8 bg-orange rounded-lg flex items-center justify-center">
                        <span class="text-white text-xs font-bold font-orbitron">{"S"}</span>
                    </div>
                    <span class="font-orbitron text-sm font-bold text-white tracking-widest">
                        {"STAR"}
                        <span class="text-orange">{"BOUND"}</span>
                    </span>
                </div>
            </Link<Route>>

            <div class="flex items-center gap-1">
                <Link<Route> to={Route::Catalog}>
                    <span class="px-4 py-2 rounded-xl text-muted hover:text-white hover:bg-navy3 text-sm font-medium transition-all duration-200 cursor-pointer">
                        {"Catalog"}
                    </span>
                </Link<Route>>
            </div>

            <div class="flex items-center gap-3">
                <Link<Route> to={Route::Cart}>
                    <div class="relative flex items-center gap-2 px-4 py-2 bg-navy3 border border-border rounded-xl text-sm font-medium text-muted hover:border-orange hover:text-orange transition-all duration-200 cursor-pointer">
                        {"Cart"}
                        if item_count > 0 {
                            <span class="bg-orange text-white text-xs font-bold px-1.5 py-0.5 rounded-full">
                                { item_count }
                            </span>
                        }
                    </div>
                </Link<Route>>

                if auth.is_authenticated() {
                    <Link<Route> to={Route::Profile}>
                        <span class="btn-primary text-sm cursor-pointer">
                            { auth.user.as_ref().map(|u| u.name.clone()).unwrap_or_default() }
                        </span>
                    </Link<Route>>
                } else {
                    <Link<Route> to={Route::Login}>
                        <span class="btn-ghost text-sm cursor-pointer">{"Sign in"}</span>
                    </Link<Route>>
                    <Link<Route> to={Route::Register}>
                        <span class="btn-primary text-sm cursor-pointer">{"Sign up"}</span>
                    </Link<Route>>
                }
            </div>
        </nav>
    }
}