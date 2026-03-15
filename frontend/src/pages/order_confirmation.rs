use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::components::ui::spinner::{Spinner, SpinnerSize};
use crate::context::auth::AuthContext;
use crate::services::orders::OrderService;
use crate::types::Order;
use crate::route::Route;

#[derive(Properties, PartialEq)]
pub struct OrderConfirmationProps {
    pub id: String,
}

#[function_component(OrderConfirmation)]
pub fn order_confirmation(props: &OrderConfirmationProps) -> Html {
    let auth    = use_context::<AuthContext>().expect("AuthContext not found");
    let order   = use_state(|| Option::<Order>::None);
    let loading = use_state(|| true);
    let error   = use_state(|| Option::<String>::None);

    {
        let order   = order.clone();
        let loading = loading.clone();
        let error   = error.clone();
        let id      = props.id.clone();
        let token   = auth.token.clone();

        use_effect_with(id.clone(), move |_| {
            spawn_local(async move {
                match token {
                    None => {
                        error.set(Some("Not authenticated".to_string()));
                        loading.set(false);
                    }
                    Some(t) => {
                        match OrderService::get(&id, &t).await {
                            Ok(o)  => order.set(Some(o)),
                            Err(e) => error.set(Some(e)),
                        }
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    if *loading {
        return html! {
            <div class="min-h-screen bg-navy flex items-center justify-center">
                <Spinner size={SpinnerSize::Lg} />
            </div>
        };
    }

    if let Some(err) = (*error).clone() {
        return html! {
            <div class="min-h-screen bg-navy flex items-center justify-center px-6">
                <div class="text-center">
                    <p class="font-orbitron text-orange text-lg mb-2">{"Could not load order"}</p>
                    <p class="font-exo text-muted text-sm mb-6">{ err }</p>
                    <Link<Route> to={Route::Orders}>
                        <button class="btn-ghost">{"View all orders"}</button>
                    </Link<Route>>
                </div>
            </div>
        };
    }

    let Some(o) = (*order).clone() else {
        return html! {
            <div class="min-h-screen bg-navy flex items-center justify-center">
                <p class="font-orbitron text-muted">{"Order not found"}</p>
            </div>
        };
    };

    html! {
        <div class="min-h-screen bg-navy">
            <div class="max-w-2xl mx-auto px-4 py-16">

                // Success header
                <div class="text-center mb-12 animate-fade-up">
                    <div class="w-20 h-20 bg-green-500/10 border-2 border-green-500/30
                                rounded-full flex items-center justify-center mx-auto mb-6">
                        <svg width="36" height="36" viewBox="0 0 24 24" fill="none"
                             stroke="#4ade80" stroke-width="2" stroke-linecap="round">
                            <polyline points="20 6 9 17 4 12"/>
                        </svg>
                    </div>

                    <h1 class="font-orbitron text-3xl font-bold text-white mb-3">
                        {"Order confirmed"}
                    </h1>
                    <p class="font-exo text-muted text-base">
                        {"Your order has been placed and is being processed."}
                    </p>
                    <div class="inline-flex items-center gap-2 mt-4 px-4 py-2
                                bg-navy2 border border-border rounded-xl">
                        <span class="font-exo text-sm text-muted">{"Order ID"}</span>
                        <span class="font-orbitron text-sm text-orange">{ &o.id }</span>
                    </div>
                </div>

                // Order details card
                <div class="card-static p-6 mb-6 animate-fade-up" style="animation-delay:100ms">

                    // Items
                    <p class="label-mono mb-4">{"Items ordered"}</p>
                    <div class="space-y-4 mb-6">
                        { for o.items.iter().map(|item| html! {
                            <div class="flex items-center gap-4 py-3 border-b border-border last:border-0">
                                <div class="w-24 h-24 bg-navy3 border border-border rounded-xl
                                            flex items-center justify-center flex-shrink-0">
                                    <span class="font-orbitron text-xs font-bold text-border">
                                        { &item.product_type.to_uppercase()[..4.min(item.product_type.len())] }
                                    </span>
                                </div>
                                <div class="flex-1 min-w-0">
                                    <p class="font-exo text-sm font-medium text-white truncate">
                                        { &item.product_name }
                                    </p>
                                    <p class="font-exo text-xs text-muted">
                                        { format!("Qty: {}", item.quantity) }
                                    </p>
                                </div>
                                <span class="font-orbitron text-sm text-white flex-shrink-0">
                                    { format_price(item.line_total) }
                                </span>
                            </div>
                        })}
                    </div>

                    // Totals
                    <div class="space-y-2 pt-2">
                        <div class="flex justify-between">
                            <span class="font-exo text-sm text-muted">{"Subtotal"}</span>
                            <span class="font-orbitron text-sm text-white">
                                { format_price(o.subtotal) }
                            </span>
                        </div>
                        <div class="flex justify-between">
                            <span class="font-exo text-sm text-muted">{"Shipping"}</span>
                            <span class="font-orbitron text-sm text-white">
                                { format_price(o.shipping_cost) }
                            </span>
                        </div>
                        <div class="flex justify-between pt-3 border-t border-border">
                            <span class="font-exo text-base font-semibold text-white">{"Total"}</span>
                            <span class="font-orbitron text-xl font-bold text-orange">
                                { format_price(o.total) }
                            </span>
                        </div>
                    </div>
                </div>

                // Shipping address
                <div class="card-static p-6 mb-6 animate-fade-up" style="animation-delay:200ms">
                    <p class="label-mono mb-4">{"Shipping to"}</p>
                    <div class="font-exo text-sm text-muted space-y-1">
                        if !o.shipping_address.facility_name.is_empty() {
                            <p class="text-white font-medium">{ &o.shipping_address.facility_name }</p>
                        }
                        if !o.shipping_address.site_code.is_empty() {
                            <p>{ format!("Site: {}", &o.shipping_address.site_code) }</p>
                        }
                        <p>{ &o.shipping_address.address_line_1 }</p>
                        if let Some(line2) = &o.shipping_address.address_line_2 {
                            <p>{ line2 }</p>
                        }
                        <p>{ format!("{}, {}", &o.shipping_address.city, &o.shipping_address.postal_code) }</p>
                        <p>{ &o.shipping_address.country }</p>
                    </div>
                </div>

                // Portfolio notice
                <div class="bg-navy3 border border-border rounded-xl px-4 py-3 mb-8
                             animate-fade-up" style="animation-delay:300ms">
                    <p class="font-exo text-xs text-dim text-center">
                        {"Note: This is a portfolio project — no real order has been placed and "}
                        {"no confirmation email has been sent. This is for demonstration purposes only."}
                    </p>
                </div>

                // Actions
                <div class="flex flex-col sm:flex-row gap-3 animate-fade-up"
                     style="animation-delay:400ms">
                    <Link<Route> to={Route::Orders}>
                        <button class="btn-ghost w-full sm:w-auto px-6 py-3">
                            {"View all orders"}
                        </button>
                    </Link<Route>>
                    <Link<Route> to={Route::Catalog}>
                        <button class="btn-primary w-full sm:w-auto px-6 py-3">
                            {"Continue shopping"}
                        </button>
                    </Link<Route>>
                </div>
            </div>
        </div>
    }
}

fn format_price(price: f64) -> String {
    if price >= 1_000_000.0 { format!("${:.1}M", price / 1_000_000.0) }
    else if price >= 1_000.0 { format!("${:.0}K", price / 1_000.0) }
    else { format!("${:.0}", price) }
}