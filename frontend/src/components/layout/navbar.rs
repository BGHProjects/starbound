use yew::prelude::*;
use yew_router::prelude::*;
use crate::route::Route;
use crate::context::auth::AuthContext;
use crate::context::cart::CartContext;

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let auth       = use_context::<AuthContext>().expect("AuthContext not found");
    let cart       = use_context::<CartContext>().expect("CartContext not found");
    let item_count = cart.item_count();

    html! {
        <nav class="bg-navy2 border-b border-border h-16 px-4 md:px-6 flex items-center justify-between sticky top-0 z-50">

            // Logo
            <Link<Route> to={Route::Landing}>
                <div class="flex items-center gap-2 cursor-pointer flex-shrink-0">
                    <div class="w-8 h-8 bg-orange rounded-lg flex items-center justify-center">
                        <span class="text-white font-orbitron font-bold text-xs">{"S"}</span>
                    </div>
                    <span class="font-orbitron text-sm font-bold text-white tracking-widest hidden sm:block">
                        {"STAR"}
                        <span class="text-orange">{"BOUND"}</span>
                    </span>
                </div>
            </Link<Route>>

            // Right actions
            <div class="flex items-center gap-2 ml-auto">

                // Cart
                <Link<Route> to={Route::Cart}>
                    <div class="relative flex items-center gap-2 px-3 py-2 bg-navy3 border border-border rounded-xl text-sm font-medium text-muted hover:border-orange hover:text-orange transition-all duration-200 cursor-pointer">
                        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M1 1h2l2.4 9.6A2 2 0 0 0 7.3 12H13a2 2 0 0 0 1.9-1.4L16 5H4"/>
                            <circle cx="7" cy="15" r="1"/>
                            <circle cx="13" cy="15" r="1"/>
                        </svg>
                        <span class="hidden sm:inline">{"Cart"}</span>
                        if item_count > 0 {
                            <span class="bg-orange text-white text-xs font-bold px-1.5 py-0.5 rounded-full leading-none">
                                { item_count }
                            </span>
                        }
                    </div>
                </Link<Route>>

                // Auth state
                if auth.is_authenticated() {
                    <Link<Route> to={Route::Profile}>
                        <div class="flex items-center gap-2 px-3 py-2 bg-orange/10 border border-orange/30 rounded-xl cursor-pointer hover:bg-orange/20 transition-all duration-200 max-w-32">
                            <div class="w-5 h-5 bg-orange rounded-full flex items-center justify-center flex-shrink-0">
                                <span class="text-white font-bold text-xs">
                                    { auth.user.as_ref()
                                        .and_then(|u| u.name.chars().next())
                                        .map(|c| c.to_uppercase().to_string())
                                        .unwrap_or_default() }
                                </span>
                            </div>
                            <span class="font-exo text-sm text-orange font-medium truncate hidden sm:block">
                                { auth.user.as_ref()
                                    .map(|u| u.name.split_whitespace().next().unwrap_or(&u.name).to_string())
                                    .unwrap_or_default() }
                            </span>
                        </div>
                    </Link<Route>>
                } else {
                    <Link<Route> to={Route::Login}>
                        <span class="btn-ghost text-sm cursor-pointer px-3 py-2">{"Sign in"}</span>
                    </Link<Route>>
                    <Link<Route> to={Route::Register}>
                        <span class="btn-primary text-sm cursor-pointer px-3 py-2">{"Sign up"}</span>
                    </Link<Route>>
                }
            </div>
        </nav>
    }
}