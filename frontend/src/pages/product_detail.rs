use crate::components::layout::chatbot_widget::ChatbotWidget;
use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::components::ui::spinner::{Spinner, SpinnerSize};
use crate::context::cart::{CartContext, CartAction};
use crate::services::products::ProductService;
use crate::types::{Product, ProductListItem};
use crate::route::Route;

#[derive(Properties, PartialEq)]
pub struct ProductDetailProps {
    pub id: String,
}

#[function_component(ProductDetail)]
pub fn product_detail(props: &ProductDetailProps) -> Html {
    let product  = use_state(|| Option::<Product>::None);
    let loading  = use_state(|| true);
    let error    = use_state(|| Option::<String>::None);
    let added    = use_state(|| false);
    let cart     = use_context::<CartContext>().expect("CartContext not found");

    // Fetch product on mount
    {
        let product = product.clone();
        let loading = loading.clone();
        let error   = error.clone();
        let id      = props.id.clone();

        use_effect_with(id.clone(), move |_| {
            spawn_local(async move {
                match ProductService::get(&id).await {
                    Ok(p)  => product.set(Some(p)),
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
            });
            || ()
        });
    }

    // Loading state
    if *loading {
        return html! {
            <div class="min-h-screen bg-navy flex items-center justify-center">
                <Spinner size={SpinnerSize::Lg} />
            </div>
        };
    }

    // Error state
    if let Some(err) = (*error).clone() {
        return html! {
            <div class="min-h-screen bg-navy flex items-center justify-center px-6">
                <div class="text-center">
                    <p class="font-orbitron text-orange text-lg mb-2">{"Failed to load product"}</p>
                    <p class="font-exo text-muted text-sm">{ err }</p>
                    <Link<Route> to={Route::Catalog}>
                        <button class="btn-ghost mt-6">{"Back to catalog"}</button>
                    </Link<Route>>
                </div>
            </div>
        };
    }

    // Product not found
    let Some(p) = (*product).clone() else {
        return html! {
            <div class="min-h-screen bg-navy flex items-center justify-center">
                <p class="font-orbitron text-muted">{"Product not found"}</p>
            </div>
        };
    };

    let stock_badge = if !p.in_stock {
        html! { <span class="badge-pre">{"Out of stock"}</span> }
    } else if p.stock_count <= 3 {
        html! { <span class="badge-low">{ format!("{} left in stock", p.stock_count) }</span> }
    } else {
        html! { <span class="badge-stock">{ format!("{} in stock", p.stock_count) }</span> }
    };

    let on_add = {
        let cart    = cart.clone();
        let added   = added.clone();
        let product = p.clone();

        Callback::from(move |_: MouseEvent| {
            let list_item = ProductListItem {
                id:           product.id.clone(),
                name:         product.name.clone(),
                group:        product.group.clone(),
                product_type: product.product_type.clone(),
                price:        product.price,
                image_url:    product.image_url.clone(),
                in_stock:     product.in_stock,
                stock_count:  product.stock_count,
            };
            cart.dispatch(CartAction::AddItem(list_item));
            added.set(true);

            let added = added.clone();
            gloo_timers::callback::Timeout::new(1500, move || {
                added.set(false);
            }).forget();
        })
    };

    let group_label = match p.group.as_str() {
        "structural" => "Structural",
        "guidance"   => "Guidance",
        "payload"    => "Payload",
        "propulsion" => "Propulsion",
        other        => other,
    };

    html! {
        <div class="min-h-screen bg-navy">
            <div class="max-w-6xl mx-auto px-6 py-10">

                // Breadcrumb
                <div class="flex items-center gap-2 font-exo text-sm text-muted mb-8 animate-fade-in">
                    <Link<Route> to={Route::Landing}>
                        <span class="hover:text-orange transition-colors cursor-pointer">{"Home"}</span>
                    </Link<Route>>
                    <span>{"/"}</span>
                    <Link<Route> to={Route::Catalog}>
                        <span class="hover:text-orange transition-colors cursor-pointer">{"Catalog"}</span>
                    </Link<Route>>
                    <span>{"/"}</span>
                    <span class="text-white">{ &p.name }</span>
                </div>

                // Main content
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-10 animate-fade-up">

                    // ── Left — image ─────────────────────────────
                    <div>
                        <div class="bg-navy2 border border-border rounded-2xl aspect-square flex items-center justify-center relative overflow-hidden">
                        
                            <img src={format!("/{}", p.image_url)} alt="product image" class="w-full h-full object-cover" />

                            // Category tag
                            <div class="absolute top-4 left-4">
                                <span class="label-mono text-xs bg-navy px-3 py-1.5 rounded-lg border border-border">
                                    { group_label }
                                </span>
                            </div>
                        </div>
                    </div>

                    // ── Right — details ──────────────────────────
                    <div class="flex flex-col">

                        // Type + name
                        <p class="label-mono mb-2">{ &p.product_type }</p>
                        <h1 class="font-orbitron text-3xl font-bold text-white mb-3 leading-tight">
                            { &p.name }
                        </h1>

                        // Stock badge
                        <div class="mb-5">
                            { stock_badge }
                        </div>

                        // Price
                        <div class="mb-6 pb-6 border-b border-border">
                            <span class="font-orbitron text-4xl font-bold text-orange">
                                { format_price(p.price) }
                            </span>
                            <span class="font-exo text-muted text-sm ml-2">{"USD"}</span>
                        </div>

                        // Attributes
                        if let Some(attrs) = &p.attributes {
                            <div class="mb-8">
                                <p class="label-mono mb-4">{"Specifications"}</p>
                                <div class="space-y-2">
                                    { for attrs.iter().map(|(key, val)| {
                                        let label = key
                                            .replace('_', " ")
                                            .split_whitespace()
                                            .map(|w| {
                                                let mut c = w.chars();
                                                match c.next() {
                                                    None => String::new(),
                                                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                            .join(" ");

                                        let value = match val {
                                            serde_json::Value::String(s) => s.clone(),
                                            serde_json::Value::Number(n) => n.to_string(),
                                            serde_json::Value::Bool(b)   => if *b { "Yes".to_string() } else { "No".to_string() },
                                            other => other.to_string(),
                                        };

                                        html! {
                                            <div class="flex items-start justify-between py-2.5 border-b border-border last:border-0">
                                                <span class="font-exo text-sm text-muted flex-shrink-0 w-1/2">
                                                    { label }
                                                </span>
                                                <span class="font-exo text-sm text-white text-right w-1/2">
                                                    { value }
                                                </span>
                                            </div>
                                        }
                                    })}
                                </div>
                            </div>
                        }

                        // Spacer
                        <div class="flex-1" />

                        // Actions
                        <div class="flex gap-3 mt-4">
                            <button
                                onclick={on_add}
                                disabled={!p.in_stock}
                                class={if *added {
                                    "flex-1 py-3 rounded-xl font-exo font-semibold text-sm bg-green-500/20 text-green-400 border border-green-500/30 transition-all duration-200"
                                } else if p.in_stock {
                                    "flex-1 py-3 rounded-xl font-exo font-semibold text-sm btn-primary transition-all duration-200"
                                } else {
                                    "flex-1 py-3 rounded-xl font-exo font-semibold text-sm bg-navy3 text-dim border border-border cursor-not-allowed opacity-50"
                                }}
                            >
                                if *added {
                                    {"Added to cart ✓"}
                                } else if p.in_stock {
                                    {"Add to cart"}
                                } else {
                                    {"Out of stock"}
                                }
                            </button>

                            <Link<Route> to={Route::Compare { id: p.id.clone() }}>
                                <button class="btn-ghost py-3 px-5 text-sm">
                                    {"Compare"}
                                </button>
                            </Link<Route>>
                        </div>

                    </div>
                </div>
            </div>
        <ChatbotWidget />
        </div>
    }
}

fn format_price(price: f64) -> String {
    if price >= 1_000_000.0 {
        format!("${:.1}M", price / 1_000_000.0)
    } else if price >= 1_000.0 {
        format!("${:.0}K", price / 1_000.0)
    } else {
        format!("${:.0}", price)
    }
}