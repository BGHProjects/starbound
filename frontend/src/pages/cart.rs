use yew::prelude::*;
use yew_router::prelude::*;
use crate::context::cart::{CartContext, CartAction};
use crate::context::auth::AuthContext;
use crate::route::Route;

#[function_component(Cart)]
pub fn cart() -> Html {
    let cart = use_context::<CartContext>().expect("CartContext not found");
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    let items      = cart.items.clone();
    let is_empty   = items.is_empty();
    let subtotal   = cart.total();
    let shipping   = if is_empty { 0.0 } else { 2500.0 };
    let total      = subtotal + shipping;

    html! {
        <div class="min-h-screen bg-navy">
            <div class="max-w-6xl mx-auto px-4 py-10">

                // Header
                <div class="mb-8 animate-fade-in">
                    <p class="label-mono mb-1">{"Your order"}</p>
                    <h1 class="font-orbitron text-2xl font-bold text-white">{"Cart"}</h1>
                </div>

                if is_empty {
                    // Empty state
                    <div class="card-static p-16 text-center animate-fade-up">
                        <div class="w-16 h-16 bg-navy3 border border-border rounded-2xl flex items-center justify-center mx-auto mb-6">
                            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="#3a4e70" stroke-width="1.5">
                                <path d="M6 2L3 6v14a2 2 0 002 2h14a2 2 0 002-2V6l-3-4z"/>
                                <line x1="3" y1="6" x2="21" y2="6"/>
                                <path d="M16 10a4 4 0 01-8 0"/>
                            </svg>
                        </div>
                        <p class="font-orbitron text-lg text-muted mb-2">{"Your cart is empty"}</p>
                        <p class="font-exo text-sm text-dim mb-8">
                            {"Browse our catalog to find rocket components"}
                        </p>
                        <Link<Route> to={Route::Catalog}>
                            <button class="btn-primary">{"Browse catalog"}</button>
                        </Link<Route>>
                    </div>
                } else {
                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8 animate-fade-up">

                        // ── Item list ─────────────────────────────
                        <div class="lg:col-span-2 flex flex-col gap-4">
                            { for items.iter().map(|item| {
                                let product_id = item.product.id.clone();

                                let on_remove = {
                                    let cart = cart.clone();
                                    let id   = product_id.clone();
                                    Callback::from(move |_: MouseEvent| {
                                        cart.dispatch(CartAction::RemoveItem(id.clone()));
                                    })
                                };

                                let on_decrease = {
                                    let cart = cart.clone();
                                    let id   = product_id.clone();
                                    let qty  = item.quantity;
                                    Callback::from(move |_: MouseEvent| {
                                        cart.dispatch(CartAction::UpdateQuantity(id.clone(), qty - 1));
                                    })
                                };

                                let on_increase = {
                                    let cart = cart.clone();
                                    let id   = product_id.clone();
                                    let qty  = item.quantity;
                                    Callback::from(move |_: MouseEvent| {
                                        cart.dispatch(CartAction::UpdateQuantity(id.clone(), qty + 1));
                                    })
                                };

                                html! {
                                    <div class="card-static p-5 flex items-center gap-5">

                                        // Image placeholder
                                        <Link<Route> to={Route::ProductDetail { id: product_id.clone() }}>
                                            <div class="w-20 h-20 bg-navy3 border border-border rounded-xl flex items-center justify-center flex-shrink-0 cursor-pointer hover:border-orange transition-colors duration-200">
                                                <span class="font-orbitron text-xs font-bold text-border">
                                                    { &item.product.product_type.to_uppercase()[..4.min(item.product.product_type.len())] }
                                                </span>
                                            </div>
                                        </Link<Route>>

                                        // Info
                                        <div class="flex-1 min-w-0">
                                            <p class="label-mono text-xs mb-1">{ &item.product.product_type }</p>
                                            <Link<Route> to={Route::ProductDetail { id: product_id.clone() }}>
                                                <h3 class="font-exo font-semibold text-white text-sm leading-snug hover:text-orange transition-colors cursor-pointer mb-2">
                                                    { &item.product.name }
                                                </h3>
                                            </Link<Route>>
                                            <p class="font-orbitron text-sm font-bold text-orange">
                                                { format_price(item.line_total()) }
                                            </p>
                                        </div>

                                        // Quantity controls
                                        <div class="flex items-center gap-2 flex-shrink-0">
                                            <button
                                                onclick={on_decrease}
                                                class="w-8 h-8 rounded-lg bg-navy3 border border-border text-muted
                                                       hover:border-orange hover:text-orange transition-all duration-150
                                                       flex items-center justify-center font-bold text-sm"
                                            >
                                                {"−"}
                                            </button>
                                            <span class="font-orbitron text-sm text-white w-6 text-center">
                                                { item.quantity }
                                            </span>
                                            <button
                                                onclick={on_increase}
                                                class="w-8 h-8 rounded-lg bg-navy3 border border-border text-muted
                                                       hover:border-orange hover:text-orange transition-all duration-150
                                                       flex items-center justify-center font-bold text-sm"
                                            >
                                                {"+"}
                                            </button>
                                        </div>

                                        // Remove
                                        <button
                                            onclick={on_remove}
                                            class="w-8 h-8 rounded-lg bg-navy3 border border-border text-dim
                                                   hover:border-red-500/50 hover:text-red-400 transition-all duration-150
                                                   flex items-center justify-center flex-shrink-0"
                                        >
                                            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5">
                                                <line x1="1" y1="1" x2="11" y2="11"/>
                                                <line x1="11" y1="1" x2="1" y2="11"/>
                                            </svg>
                                        </button>
                                    </div>
                                }
                            })}

                            // Clear cart
                            <div class="flex justify-end mt-2">
                                <button
                                    onclick={Callback::from({
                                        let cart = cart.clone();
                                        move |_: MouseEvent| cart.dispatch(CartAction::Clear)
                                    })}
                                    class="font-exo text-xs text-dim hover:text-red-400 transition-colors"
                                >
                                    {"Clear cart"}
                                </button>
                            </div>
                        </div>

                        // ── Order summary ─────────────────────────
                        <div class="lg:col-span-1">
                            <div class="card-static p-6 sticky top-24">
                                <p class="label-mono mb-5">{"Order summary"}</p>

                                // Line items summary
                                <div class="space-y-3 mb-5">
                                    { for items.iter().map(|item| html! {
                                        <div class="flex items-start justify-between gap-3">
                                            <span class="font-exo text-xs text-muted leading-relaxed flex-1">
                                                { format!("{} × {}", item.quantity, &item.product.name) }
                                            </span>
                                            <span class="font-orbitron text-xs text-white flex-shrink-0">
                                                { format_price(item.line_total()) }
                                            </span>
                                        </div>
                                    })}
                                </div>

                                <div class="border-t border-border pt-4 space-y-3 mb-6">
                                    <div class="flex justify-between">
                                        <span class="font-exo text-sm text-muted">{"Subtotal"}</span>
                                        <span class="font-orbitron text-sm text-white">
                                            { format_price(subtotal) }
                                        </span>
                                    </div>
                                    <div class="flex justify-between">
                                        <span class="font-exo text-sm text-muted">{"Shipping"}</span>
                                        <span class="font-orbitron text-sm text-white">
                                            { format_price(shipping) }
                                        </span>
                                    </div>
                                    <div class="flex justify-between pt-3 border-t border-border">
                                        <span class="font-exo text-base font-semibold text-white">{"Total"}</span>
                                        <span class="font-orbitron text-lg font-bold text-orange">
                                            { format_price(total) }
                                        </span>
                                    </div>
                                </div>

                                // CTA
                                if auth.is_authenticated() {
                                    <Link<Route> to={Route::Checkout}>
                                        <button class="btn-primary w-full py-3 text-sm">
                                            {"Proceed to checkout"}
                                        </button>
                                    </Link<Route>>
                                } else {
                                    <div class="space-y-3">
                                        <Link<Route> to={Route::Login}>
                                            <button class="btn-primary w-full py-3 text-sm">
                                                {"Sign in to checkout"}
                                            </button>
                                        </Link<Route>>
                                        <p class="font-exo text-xs text-dim text-center">
                                            {"You need an account to place an order"}
                                        </p>
                                    </div>
                                }

                                // Continue shopping
                                <Link<Route> to={Route::Catalog}>
                                    <button class="btn-ghost w-full py-2.5 text-sm mt-3">
                                        {"Continue shopping"}
                                    </button>
                                </Link<Route>>
                            </div>
                        </div>
                    </div>
                }
            </div>
        </div>
    }
}

fn format_price(price: f64) -> String {
    if price >= 1_000_000.0 { format!("${:.1}M", price / 1_000_000.0) }
    else if price >= 1_000.0 { format!("${:.0}K", price / 1_000.0) }
    else { format!("${:.0}", price) }
}