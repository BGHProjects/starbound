use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::components::ui::spinner::{Spinner, SpinnerSize};
use crate::context::auth::{AuthContext, AuthAction};
use crate::services::orders::OrderService;
use crate::types::Order;
use crate::route::Route;

#[function_component(Profile)]
pub fn profile() -> Html {
    let auth      = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();
    let orders    = use_state(|| Vec::<Order>::new());
    let loading   = use_state(|| true);

    if !auth.is_authenticated() {
        navigator.push(&Route::Login);
        return html! {};
    }

    let user = auth.user.clone().unwrap();

    {
        let orders  = orders.clone();
        let loading = loading.clone();
        let token   = auth.token.clone().unwrap_or_default();

        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(resp) = OrderService::list(&token, 1).await {
                    orders.set(resp.data);
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_logout = {
        let auth      = auth.clone();
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| {
            auth.dispatch(AuthAction::Logout);
            navigator.push(&Route::Landing);
        })
    };

    // Initials from name
    let initials = user.name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();

    // Stats derived from orders
    let total_orders  = (*orders).len();
    let total_spent: f64 = (*orders).iter().map(|o| o.total).sum();
    let active_orders = (*orders).iter()
        .filter(|o| !matches!(o.status.as_str(), "delivered" | "cancelled" | "refunded"))
        .count();

    html! {
        <div class="min-h-screen bg-navy">
            <div class="max-w-4xl mx-auto px-4 py-10">

                // Header
                <div class="mb-8 animate-fade-in">
                    <p class="label-mono mb-1">{"Your account"}</p>
                    <h1 class="font-orbitron text-2xl font-bold text-white">{"Profile"}</h1>
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">

                    // ── Left — user card ──────────────────────────
                    <div class="lg:col-span-1 space-y-4">

                        // Avatar + info
                        <div class="card-static p-6 text-center animate-fade-up">
                            <div class="w-20 h-20 bg-orange/10 border-2 border-orange/30
                                        rounded-full flex items-center justify-center mx-auto mb-4">
                                <span class="font-orbitron text-2xl font-bold text-orange">
                                    { &initials }
                                </span>
                            </div>
                            <h2 class="font-orbitron text-lg font-bold text-white mb-1">
                                { &user.name }
                            </h2>
                            <p class="font-exo text-sm text-muted mb-6">
                                { &user.email }
                            </p>
                            <p class="font-exo text-xs text-dim mb-6">
                                { format!("Member since {}", format_date(&user.created_at)) }
                            </p>
                            <button
                                onclick={on_logout}
                                class="w-full btn-ghost text-sm py-2.5
                                       border-red-500/20 text-red-400/70
                                       hover:border-red-500/50 hover:text-red-400
                                       transition-all duration-200"
                            >
                                {"Sign out"}
                            </button>
                        </div>

                        // Stats
                        <div class="card-static p-6 animate-fade-up" style="animation-delay:100ms">
                            <p class="label-mono mb-4">{"Account stats"}</p>
                            <div class="space-y-4">
                                <div class="flex justify-between items-baseline">
                                    <span class="font-exo text-sm text-muted">{"Total orders"}</span>
                                    <span class="font-orbitron text-lg font-bold text-white">
                                        { total_orders }
                                    </span>
                                </div>
                                <div class="flex justify-between items-baseline">
                                    <span class="font-exo text-sm text-muted">{"Active orders"}</span>
                                    <span class="font-orbitron text-lg font-bold text-orange">
                                        { active_orders }
                                    </span>
                                </div>
                                <div class="flex justify-between items-baseline border-t border-border pt-4">
                                    <span class="font-exo text-sm text-muted">{"Total spent"}</span>
                                    <span class="font-orbitron text-lg font-bold text-orange">
                                        { format_price(total_spent) }
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>

                    // ── Right — recent orders ─────────────────────
                    <div class="lg:col-span-2 animate-fade-up" style="animation-delay:150ms">
                        <div class="card-static p-6">
                            <div class="flex items-center justify-between mb-5">
                                <p class="label-mono">{"Recent orders"}</p>
                                <Link<Route> to={Route::Orders}>
                                    <span class="font-exo text-xs text-muted hover:text-orange
                                                 transition-colors cursor-pointer">
                                        {"View all →"}
                                    </span>
                                </Link<Route>>
                            </div>

                            if *loading {
                                <div class="flex justify-center py-10">
                                    <Spinner size={SpinnerSize::Md} />
                                </div>
                            } else if (*orders).is_empty() {
                                <div class="text-center py-10">
                                    <p class="font-exo text-sm text-muted mb-4">
                                        {"No orders yet"}
                                    </p>
                                    <Link<Route> to={Route::Catalog}>
                                        <button class="btn-primary text-sm px-5 py-2">
                                            {"Browse catalog"}
                                        </button>
                                    </Link<Route>>
                                </div>
                            } else {
                                <div class="space-y-3">
                                    { for (*orders).iter().take(5).map(|order| {
                                        let status_class = status_color(&order.status);
                                        let order_id     = order.id.clone();
                                        html! {
                                            <div
                                                class="flex items-center gap-4 p-3 rounded-xl
                                                       bg-navy3 hover:bg-navy4 border border-border
                                                       hover:border-orange transition-all duration-150
                                                       cursor-pointer"
                                                onclick={
                                                    let navigator = navigator.clone();
                                                    let id        = order_id.clone();
                                                    Callback::from(move |_: MouseEvent| {
                                                        navigator.push(&Route::OrderDetail { id: id.clone() });
                                                    })
                                                }
                                            >
                                                // Order ref
                                                <span class="font-orbitron text-xs text-orange flex-shrink-0">
                                                    { format!("#{}", &order.id[..8.min(order.id.len())]) }
                                                </span>

                                                // Item names
                                                <span class="font-exo text-xs text-white flex-1 truncate">
                                                    { order.items.iter()
                                                        .map(|i| i.product_name.clone())
                                                        .collect::<Vec<_>>()
                                                        .join(", ") }
                                                </span>

                                                // Status
                                                <span class={format!(
                                                    "font-exo text-xs px-2 py-0.5 rounded-full border flex-shrink-0 {}",
                                                    status_class
                                                )}>
                                                    { format_status(&order.status) }
                                                </span>

                                                // Total
                                                <span class="font-orbitron text-xs text-white flex-shrink-0">
                                                    { format_price(order.total) }
                                                </span>
                                            </div>
                                        }
                                    })}
                                </div>
                            }
                        </div>
                    </div>
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