use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::components::ui::spinner::{Spinner, SpinnerSize};
use crate::context::auth::AuthContext;
use crate::services::orders::OrderService;
use crate::types::Order;
use crate::route::Route;

#[function_component(Orders)]
pub fn orders() -> Html {
    let auth      = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();
    let orders    = use_state(|| Vec::<Order>::new());
    let loading   = use_state(|| true);
    let error     = use_state(|| Option::<String>::None);
    let page      = use_state(|| 1i32);
    let total     = use_state(|| 0i32);

    if !auth.is_authenticated() {
        navigator.push(&Route::Login);
        return html! {};
    }

    {
        let orders  = orders.clone();
        let loading = loading.clone();
        let error   = error.clone();
        let total   = total.clone();
        let token   = auth.token.clone().unwrap_or_default();
        let p       = *page;

        use_effect_with(p, move |_| {
            loading.set(true);
            spawn_local(async move {
                match OrderService::list(&token, p).await {
                    Ok(resp) => {
                        orders.set(resp.data);
                        total.set(resp.total);
                    }
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
            });
            || ()
        });
    }

    let total_pages = ((*total as f32) / 20.0).ceil() as i32;

    html! {
        <div class="min-h-screen bg-navy">
            <div class="max-w-4xl mx-auto px-4 py-10">

                // Header
                <div class="mb-8 animate-fade-in">
                    <p class="label-mono mb-1">{"Your account"}</p>
                    <h1 class="font-orbitron text-2xl font-bold text-white">{"Order history"}</h1>
                </div>

                if *loading {
                    <div class="flex justify-center py-24">
                        <Spinner size={SpinnerSize::Lg} />
                    </div>
                } else if let Some(err) = (*error).clone() {
                    <div class="bg-red-500/10 border border-red-500/25 rounded-xl px-4 py-3">
                        <p class="text-red-400 font-exo text-sm">{ err }</p>
                    </div>
                } else if (*orders).is_empty() {
                    <div class="card-static p-16 text-center animate-fade-up">
                        <div class="w-16 h-16 bg-navy3 border border-border rounded-2xl
                                    flex items-center justify-center mx-auto mb-6">
                            <svg width="28" height="28" viewBox="0 0 24 24" fill="none"
                                 stroke="#3a4e70" stroke-width="1.5">
                                <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
                                <polyline points="14 2 14 8 20 8"/>
                            </svg>
                        </div>
                        <p class="font-orbitron text-lg text-muted mb-2">{"No orders yet"}</p>
                        <p class="font-exo text-sm text-dim mb-8">
                            {"Your order history will appear here once you make a purchase"}
                        </p>
                        <Link<Route> to={Route::Catalog}>
                            <button class="btn-primary">{"Browse catalog"}</button>
                        </Link<Route>>
                    </div>
                } else {
                    <div class="flex flex-col gap-4 animate-fade-up">
                        { for (*orders).iter().enumerate().map(|(i, order)| {
                            let delay        = format!("animation-delay: {}ms", i * 50);
                            let status_class = status_color(&order.status);
                            let order_id     = order.id.clone();

                            html! {
                                <div
                                    class="opacity-0 animate-fade-up card-static p-5
                                           hover:border-orange transition-all duration-200 cursor-pointer"
                                    style={delay}
                                    onclick={
                                        let navigator = navigator.clone();
                                        let id        = order_id.clone();
                                        Callback::from(move |_: MouseEvent| {
                                            navigator.push(&Route::OrderDetail { id: id.clone() });
                                        })
                                    }
                                >
                                    <div class="flex items-start justify-between gap-4 flex-wrap">

                                        // Left — order info
                                        <div class="flex-1 min-w-0">
                                            <div class="flex items-center gap-3 mb-2 flex-wrap">
                                                <span class="font-orbitron text-sm text-orange">
                                                    { format!("#{}", &order.id[..8.min(order.id.len())]) }
                                                </span>
                                                <span class={format!(
                                                    "font-exo text-xs px-2.5 py-1 rounded-full border {}",
                                                    status_class
                                                )}>
                                                    { format_status(&order.status) }
                                                </span>
                                            </div>

                                            // Item names
                                            <p class="font-exo text-sm text-white mb-1 truncate">
                                                { order.items.iter()
                                                    .map(|i| i.product_name.clone())
                                                    .collect::<Vec<_>>()
                                                    .join(", ") }
                                            </p>

                                            <p class="font-exo text-xs text-muted">
                                                { format!("{} item{} · {}",
                                                    order.items.len(),
                                                    if order.items.len() == 1 { "" } else { "s" },
                                                    format_date(&order.created_at)
                                                )}
                                            </p>
                                        </div>

                                        // Right — total and arrow
                                        <div class="flex items-center gap-4 flex-shrink-0">
                                            <div class="text-right">
                                                <p class="font-exo text-xs text-muted mb-1">{"Total"}</p>
                                                <p class="font-orbitron text-base font-bold text-orange">
                                                    { format_price(order.total) }
                                                </p>
                                            </div>
                                            <span class="text-muted text-lg">{"→"}</span>
                                        </div>
                                    </div>
                                </div>
                            }
                        })}

                        // Pagination
                        if total_pages > 1 {
                            <div class="flex items-center justify-center gap-3 mt-6">
                                <button
                                    onclick={{
                                        let page = page.clone();
                                        let p    = *page;
                                        Callback::from(move |_: MouseEvent| {
                                            if p > 1 { page.set(p - 1); }
                                        })
                                    }}
                                    disabled={*page <= 1}
                                    class="btn-ghost text-sm px-4 py-2 disabled:opacity-40"
                                >
                                    {"← Prev"}
                                </button>
                                <span class="font-orbitron text-xs text-muted">
                                    { format!("{} / {}", *page, total_pages) }
                                </span>
                                <button
                                    onclick={{
                                        let page = page.clone();
                                        let p    = *page;
                                        Callback::from(move |_: MouseEvent| {
                                            if p < total_pages { page.set(p + 1); }
                                        })
                                    }}
                                    disabled={*page >= total_pages}
                                    class="btn-ghost text-sm px-4 py-2 disabled:opacity-40"
                                >
                                    {"Next →"}
                                </button>
                            </div>
                        }
                    </div>
                }
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
    // Parse "2024-01-15T10:30:00Z" into "15 Jan 2024"
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