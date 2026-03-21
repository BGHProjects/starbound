use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::components::ui::spinner::{Spinner, SpinnerSize};
use crate::context::auth::AuthContext;
use crate::services::orders::OrderService;
use crate::types::Order;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};
use js_sys::{Array, Uint8Array};
use crate::route::Route;

#[derive(Properties, PartialEq)]
pub struct OrderDetailProps {
    pub id: String,
}

#[function_component(OrderDetail)]
pub fn order_detail(props: &OrderDetailProps) -> Html {
    let auth      = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();
    let order     = use_state(|| Option::<Order>::None);
    let loading   = use_state(|| true);
    let error     = use_state(|| Option::<String>::None);
    let cancelling = use_state(|| false);
    let cancelled  = use_state(|| false);

    if !auth.is_authenticated() {
        navigator.push(&Route::Login);
        return html! {};
    }

    {
        let order   = order.clone();
        let loading = loading.clone();
        let error   = error.clone();
        let id      = props.id.clone();
        let token   = auth.token.clone().unwrap_or_default();

        use_effect_with(id.clone(), move |_| {
            spawn_local(async move {
                match OrderService::get(&id, &token).await {
                    Ok(o)  => order.set(Some(o)),
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
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
                        <button class="btn-ghost">{"Back to orders"}</button>
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

    let can_cancel = matches!(
        o.status.as_str(),
        "pending" | "payment_processing" | "payment_failed" | "confirmed"
    );

    let on_cancel = {
        let order      = order.clone();
        let cancelling = cancelling.clone();
        let cancelled  = cancelled.clone();
        let error      = error.clone();
        let id         = o.id.clone();
        let token      = auth.token.clone().unwrap_or_default();

        Callback::from(move |_: MouseEvent| {
            let order      = order.clone();
            let cancelling = cancelling.clone();
            let cancelled  = cancelled.clone();
            let error      = error.clone();
            let id         = id.clone();
            let token      = token.clone();

            cancelling.set(true);
            spawn_local(async move {
                match OrderService::cancel(&id, &token).await {
                    Ok(updated) => {
                        order.set(Some(updated));
                        cancelled.set(true);
                    }
                    Err(e) => error.set(Some(e)),
                }
                cancelling.set(false);
            });
        })
    };

    let status_class = status_color(&o.status);

    let on_download = {
        let token    = auth.token.clone().unwrap_or_default();
        let order_id = o.id.clone();
        Callback::from(move |_: MouseEvent| {
            let token    = token.clone();
            let order_id = order_id.clone();
            spawn_local(async move {
                let url = format!("http://localhost:8000/api/orders/{}/receipt", order_id);
                let resp = gloo_net::http::Request::get(&url)
                    .header("Authorization", &format!("Bearer {}", token))
                    .send()
                    .await;
                if let Ok(resp) = resp {
                    if let Ok(bytes) = resp.binary().await {
                        let uint8 = Uint8Array::from(bytes.as_slice());
                        let arr   = Array::new();
                        arr.push(&uint8.buffer());
                        let mut opts = BlobPropertyBag::new();
                        opts.type_("application/pdf");
                        if let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&arr, &opts) {
                            if let Ok(obj_url) = Url::create_object_url_with_blob(&blob) {
                                if let Some(window) = web_sys::window() {
                                    if let Some(document) = window.document() {
                                        if let Ok(el) = document.create_element("a") {
                                            let a = el.unchecked_into::<HtmlAnchorElement>();
                                            a.set_href(&obj_url);
                                            a.set_download(&format!("starbound-receipt-{}.pdf", &order_id[..8]));
                                            a.click();
                                            let _ = Url::revoke_object_url(&obj_url);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
        })
    };

    html! {
        <div class="min-h-screen bg-navy">
            <div class="max-w-3xl mx-auto px-4 py-10">

                // Breadcrumb
                <div class="flex items-center gap-2 font-exo text-sm text-muted mb-8 animate-fade-in">
                    <Link<Route> to={Route::Orders}>
                        <span class="hover:text-orange transition-colors cursor-pointer">
                            {"Orders"}
                        </span>
                    </Link<Route>>
                    <span>{"/"}</span>
                    <span class="text-white">
                        { format!("#{}", &o.id[..8.min(o.id.len())]) }
                    </span>
                </div>

                // Header
                <div class="flex items-start justify-between gap-4 mb-8 animate-fade-up flex-wrap">
                    <div>
                        <p class="label-mono mb-1">{"Order details"}</p>
                        <h1 class="font-orbitron text-2xl font-bold text-white mb-2">
                            { format!("#{}", &o.id[..8.min(o.id.len())]) }
                        </h1>
                        <div class="flex items-center gap-3 flex-wrap">
                            <span class={format!(
                                "font-exo text-xs px-2.5 py-1 rounded-full border {}",
                                status_class
                            )}>
                                { format_status(&o.status) }
                            </span>
                            <span class="font-exo text-xs text-muted">
                                { format_date(&o.created_at) }
                            </span>
                        </div>
                    </div>

                    // Cancel button
                    if can_cancel && !*cancelled {
                        <button
                            onclick={on_cancel}
                            disabled={*cancelling}
                            class="btn-ghost text-sm px-4 py-2 border-red-500/30 text-red-400
                                   hover:border-red-500 hover:bg-red-500/10 transition-all duration-200"
                        >
                            if *cancelling {
                                <span class="flex items-center gap-2">
                                    <span class="w-3 h-3 border-2 border-red-400/30 border-t-red-400 rounded-full animate-spin"></span>
                                    {"Cancelling..."}
                                </span>
                            } else {
                                {"Cancel order"}
                            }
                        </button>
                    }
                </div>

                // Cancel success notice
                if *cancelled {
                    <div class="bg-red-500/10 border border-red-500/25 rounded-xl px-4 py-3 mb-6 animate-fade-in">
                        <p class="text-red-400 font-exo text-sm">{"Order has been cancelled."}</p>
                    </div>
                }

                // Error
                if let Some(err) = (*error).clone() {
                    <div class="bg-red-500/10 border border-red-500/25 rounded-xl px-4 py-3 mb-6 animate-fade-in">
                        <p class="text-red-400 font-exo text-sm">{ err }</p>
                    </div>
                }

                // Items
                <div class="card-static p-6 mb-6 animate-fade-up" style="animation-delay:100ms">
                    <p class="label-mono mb-4">{"Items"}</p>
                    <div class="space-y-4">
                        { for o.items.iter().map(|item| html! {
                            <div class="flex items-center gap-4 py-3 border-b border-border last:border-0">
                                <Link<Route> to={Route::ProductDetail { id: item.product_id.clone() }}>
                                    <div class="w-24 h-24 bg-navy3 border border-border rounded-xl
                                                flex items-center justify-center flex-shrink-0
                                                hover:border-orange transition-colors cursor-pointer">
                                        <img src={format!("/{}", item.image_url)} alt="product image" class="w-full h-full object-cover" />
                                    </div>
                                </Link<Route>>
                                <div class="flex-1 min-w-0">
                                    <p class="font-exo text-sm font-medium text-white">
                                        { &item.product_name }
                                    </p>
                                    <p class="font-exo text-xs text-muted">
                                        { format!("Qty: {}  ·  Unit price: {}",
                                            item.quantity,
                                            format_price(item.unit_price)) }
                                    </p>
                                </div>
                                <span class="font-orbitron text-sm text-white flex-shrink-0">
                                    { format_price(item.line_total) }
                                </span>
                            </div>
                        })}
                    </div>

                    // Totals
                    <div class="space-y-2 pt-4 mt-2 border-t border-border">
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
                    <p class="label-mono mb-4">{"Shipping address"}</p>
                    <div class="font-exo text-sm text-muted space-y-1">
                        if !o.shipping_address.facility_name.is_empty() {
                            <p class="text-white font-medium">
                                { &o.shipping_address.facility_name }
                            </p>
                        }
                        if !o.shipping_address.site_code.is_empty() {
                            <p>{ format!("Site: {}", &o.shipping_address.site_code) }</p>
                        }
                        <p>{ &o.shipping_address.address_line_1 }</p>
                        if let Some(line2) = &o.shipping_address.address_line_2 {
                            <p>{ line2 }</p>
                        }
                        <p>
                            { format!("{}, {}",
                                &o.shipping_address.city,
                                &o.shipping_address.postal_code) }
                        </p>
                        <p>{ &o.shipping_address.country }</p>
                    </div>
                </div>

                // Actions
                <div class="flex flex-col sm:flex-row gap-3 animate-fade-up"
                     style="animation-delay:300ms">
                    <Link<Route> to={Route::Orders}>
                        <button class="btn-ghost w-full sm:w-auto px-6 py-3">
                            {"← Back to orders"}
                        </button>
                    </Link<Route>>
                    <button
                        onclick={on_download.clone()}
                        class="btn-ghost w-full sm:w-auto px-6 py-3"
                    >
                        {"Download receipt"}
                    </button>
                    if o.status == "delivered" || o.status == "shipped" {
                        <Link<Route> to={Route::Refund { order_id: o.id.clone() }}>
                            <button class="btn-outline w-full sm:w-auto px-6 py-3">
                                {"Request refund"}
                            </button>
                        </Link<Route>>
                    }
                </div>
            </div>
        </div>
    }
}

fn status_color(status: &str) -> &'static str {
    match status {
        "pending"            => "bg-yellow-500/10 text-yellow-400 border-yellow-500/25",
        "payment_processing" => "bg-blue-500/10 text-blue-400 border-blue-500/25",
        "payment_failed"     => "bg-red-500/10 text-red-400 border-red-500/25",
        "confirmed"          => "bg-green-500/10 text-green-400 border-green-500/25",
        "preparing"          => "bg-orange/10 text-orange border-orange/25",
        "shipped"            => "bg-blue-500/10 text-blue-400 border-blue-500/25",
        "in_transit"         => "bg-blue-500/10 text-blue-400 border-blue-500/25",
        "delivered"          => "bg-green-500/10 text-green-400 border-green-500/25",
        "cancelled"          => "bg-red-500/10 text-red-400 border-red-500/25",
        "refund_pending"     => "bg-yellow-500/10 text-yellow-400 border-yellow-500/25",
        "refunded"           => "bg-green-500/10 text-green-400 border-green-500/25",
        _                    => "bg-navy3 text-muted border-border",
    }
}

fn format_status(status: &str) -> String {
    status.replace('_', " ")
          .split_whitespace()
          .map(|w| {
              let mut c = w.chars();
              match c.next() {
                  None    => String::new(),
                  Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
              }
          })
          .collect::<Vec<_>>()
          .join(" ")
}

fn format_date(ts: &str) -> String {
    if ts.len() < 10 { return ts.to_string(); }
    let parts: Vec<&str> = ts[..10].split('-').collect();
    if parts.len() != 3 { return ts[..10].to_string(); }
    let months = ["Jan","Feb","Mar","Apr","May","Jun",
                  "Jul","Aug","Sep","Oct","Nov","Dec"];
    let month_idx = parts[1].parse::<usize>().unwrap_or(1).saturating_sub(1);
    let month = months.get(month_idx).unwrap_or(&"");
    format!("{} {} {}", parts[2], month, parts[0])
}

fn format_price(price: f64) -> String {
    if price >= 1_000_000.0 { format!("${:.1}M", price / 1_000_000.0) }
    else if price >= 1_000.0 { format!("${:.0}K", price / 1_000.0) }
    else { format!("${:.0}", price) }
}