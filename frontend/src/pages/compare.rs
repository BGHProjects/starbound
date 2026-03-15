use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::Element;
use wasm_bindgen::JsCast;
use crate::components::ui::spinner::{Spinner, SpinnerSize};
use crate::context::cart::{CartContext, CartAction};
use crate::services::products::ProductService;
use crate::types::{Product, ProductListItem};
use crate::route::Route;
use std::collections::HashSet;

#[derive(Properties, PartialEq)]
pub struct CompareProps {
    pub id: String,
}

#[function_component(Compare)]
pub fn compare(props: &CompareProps) -> Html {
    let current      = use_state(|| Option::<Product>::None);
    let similar      = use_state(|| Vec::<Product>::new());
    let loading      = use_state(|| true);
    let error        = use_state(|| Option::<String>::None);
    let cart         = use_context::<CartContext>().expect("CartContext not found");
    let scroll_ref   = use_node_ref();
    let active_index = use_state(|| 0usize);

    {
        let current = current.clone();
        let similar = similar.clone();
        let loading = loading.clone();
        let error   = error.clone();
        let id      = props.id.clone();

        use_effect_with(id.clone(), move |_| {
            spawn_local(async move {
                match ProductService::get(&id).await {
                    Err(e) => { error.set(Some(e)); loading.set(false); return; }
                    Ok(p)  => {
                        let product_type = p.product_type.clone();
                        let product_id   = p.id.clone();
                        current.set(Some(p));
                        match ProductService::get_similar(&product_type, &product_id).await {
                            Ok(resp) => {
                                let mut full = Vec::new();
                                for item in resp.data.iter().take(2) {
                                    if let Ok(fp) = ProductService::get(&item.id).await {
                                        full.push(fp);
                                    }
                                }
                                similar.set(full);
                            }
                            Err(e) => error.set(Some(e)),
                        }
                    }
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
                    <p class="font-orbitron text-orange text-lg mb-2">{"Failed to load comparison"}</p>
                    <p class="font-exo text-muted text-sm mb-6">{ err }</p>
                    <Link<Route> to={Route::Catalog}>
                        <button class="btn-ghost">{"Back to catalog"}</button>
                    </Link<Route>>
                </div>
            </div>
        };
    }

    let Some(current_product) = (*current).clone() else {
        return html! {
            <div class="min-h-screen bg-navy flex items-center justify-center">
                <p class="font-orbitron text-muted">{"Product not found"}</p>
            </div>
        };
    };

    let similar_products = (*similar).clone();
    let total_cols       = 1 + similar_products.len();

    let mut all_keys: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let all_products: Vec<&Product> = std::iter::once(&current_product)
        .chain(similar_products.iter())
        .collect();
    for product in &all_products {
        if let Some(attrs) = &product.attributes {
            for key in attrs.keys() {
                if seen.insert(key.clone()) {
                    all_keys.push(key.clone());
                }
            }
        }
    }

    let columns: Vec<(bool, Product)> = std::iter::once((true, current_product.clone()))
        .chain(similar_products.iter().map(|p| (false, p.clone())))
        .collect();

    let get_num_range = |key: &str| -> (Option<f64>, Option<f64>) {
        let nums: Vec<f64> = columns.iter().filter_map(|(_, p)| {
            p.attributes.as_ref()?.get(key)?.as_f64()
        }).collect();
        if nums.len() < 2 { return (None, None); }
        let max = nums.iter().cloned().reduce(f64::max);
        let min = nums.iter().cloned().reduce(f64::min);
        if max == min { (None, None) } else { (max, min) }
    };

    let scroll_to = {
        let scroll_ref   = scroll_ref.clone();
        let active_index = active_index.clone();
        move |idx: usize| {
            if let Some(el) = scroll_ref.cast::<Element>() {
                // Each card is 600px wide + 24px gap
                let target_x = (600 + 24) * idx;
                let _ = js_sys::eval(&format!(
                    "document.getElementById('compare-scroll').scrollTo({{left: {}, behavior: 'smooth'}})",
                    target_x
                ));
            }
            active_index.set(idx);
        }
    };

    let scroll_prev = {
        let scroll_to    = scroll_to.clone();
        let active_index = active_index.clone();
        let total        = total_cols;
        Callback::from(move |_: MouseEvent| {
            let idx = if *active_index == 0 { total - 1 } else { *active_index - 1 };
            scroll_to(idx);
        })
    };

    let scroll_next = {
        let scroll_to    = scroll_to.clone();
        let active_index = active_index.clone();
        let total        = total_cols;
        Callback::from(move |_: MouseEvent| {
            let idx = (*active_index + 1) % total;
            scroll_to(idx);
        })
    };

    html! {
        <div class="min-h-screen bg-navy">
            <div class="max-w-6xl mx-auto px-4 py-10">

                // Breadcrumb + back
                <div class="flex items-center justify-between mb-10 animate-fade-in flex-wrap gap-4">
                    <div class="flex items-center gap-2 font-exo text-sm text-muted">
                        <Link<Route> to={Route::Landing}>
                            <span class="hover:text-orange transition-colors cursor-pointer">{"Home"}</span>
                        </Link<Route>>
                        <span>{"/"}</span>
                        <Link<Route> to={Route::Catalog}>
                            <span class="hover:text-orange transition-colors cursor-pointer">{"Catalog"}</span>
                        </Link<Route>>
                        <span>{"/"}</span>
                        <Link<Route> to={Route::ProductDetail { id: current_product.id.clone() }}>
                            <span class="hover:text-orange transition-colors cursor-pointer">
                                { &current_product.name }
                            </span>
                        </Link<Route>>
                        <span>{"/"}</span>
                        <span class="text-white">{"Compare"}</span>
                    </div>
                    <Link<Route> to={Route::ProductDetail { id: current_product.id.clone() }}>
                        <button class="btn-ghost text-sm">{"← Back to product"}</button>
                    </Link<Route>>
                </div>

                // Page title
                <div class="text-center mb-12 animate-fade-up">
                    <p class="label-mono mb-2">{"Side by side"}</p>
                    <h1 class="font-orbitron text-2xl font-bold text-white">{"Compare components"}</h1>
                </div>

                if similar_products.is_empty() {
                    <div class="card-static p-10 text-center animate-fade-up">
                        <p class="font-orbitron text-muted text-lg mb-2">{"No similar products found"}</p>
                        <p class="font-exo text-sm text-dim mb-6">
                            {"There are no other products of this type to compare against."}
                        </p>
                        <Link<Route> to={Route::ProductDetail { id: current_product.id.clone() }}>
                            <button class="btn-ghost">{"Back to product"}</button>
                        </Link<Route>>
                    </div>
                } else {
                    <div class="relative">

                        // Left nav button
                        if total_cols > 1 {
                            <button
                                onclick={scroll_prev}
                                class="lg:hidden absolute -left-3 top-1/2 -translate-y-1/2 z-10
                                       w-8 h-8 rounded-full bg-navy3/80 border border-border
                                       flex items-center justify-center text-muted text-lg
                                       hover:border-orange hover:text-orange transition-all duration-200"
                            >
                                {"‹"}
                            </button>
                        }

                        // Right nav button
                        if total_cols > 1 {
                            <button
                                onclick={scroll_next}
                                class="lg:hidden absolute -right-3 top-1/2 -translate-y-1/2 z-10
                                       w-8 h-8 rounded-full bg-navy3/80 border border-border
                                       flex items-center justify-center text-muted text-lg
                                       hover:border-orange hover:text-orange transition-all duration-200"
                            >
                                {"›"}
                            </button>
                        }

                        // Scroll container — cards are fixed 600px wide, no padding inside
                        <div
                            id="compare-scroll"
                            ref={scroll_ref}
                            class="flex gap-6 overflow-x-auto snap-x snap-mandatory scroll-smooth scrollbar-hide justify-start lg:justify-center"
                        >
                            { for columns.iter().enumerate().map(|(col_idx, (is_current, p))| {
                                let delay  = format!("animation-delay: {}ms", col_idx * 100);
                                let add_cb = {
                                    let cart    = cart.clone();
                                    let product = p.clone();
                                    Callback::from(move |_: MouseEvent| {
                                        let item = ProductListItem {
                                            id:           product.id.clone(),
                                            name:         product.name.clone(),
                                            group:        product.group.clone(),
                                            product_type: product.product_type.clone(),
                                            price:        product.price,
                                            image_url:    product.image_url.clone(),
                                            in_stock:     product.in_stock,
                                            stock_count:  product.stock_count,
                                        };
                                        cart.dispatch(CartAction::AddItem(item));
                                    })
                                };

                                html! {
                                    // Fixed width card — width matches image, no internal padding stretching
                                    <div
                                        class={if *is_current {
                                            "opacity-0 animate-fade-up flex-shrink-0 w-80 snap-center \
                                             bg-navy2 border-2 border-orange rounded-2xl overflow-hidden"
                                        } else {
                                            "opacity-0 animate-fade-up flex-shrink-0 w-80 snap-center \
                                             bg-navy2 border border-border rounded-2xl overflow-hidden"
                                        }}
                                        style={delay}
                                    >
                                        // Square image — full width of card, no padding
                                        <div class="w-full aspect-square bg-navy3 flex items-center justify-center border-b border-border">
                                            <span class="font-orbitron text-2xl font-bold text-border tracking-widest select-none">
                                                { &p.product_type.to_uppercase()[..4.min(p.product_type.len())] }
                                            </span>
                                        </div>

                                        // Card body — consistent padding
                                        <div class="p-4 flex flex-col">

                                            // Type label
                                            <p class="label-mono text-xs mb-1">{ &p.product_type }</p>

                                            // Fixed height name area — 2 lines reserved always
                                            // so cards stay aligned regardless of name length
                                            <div class="h-12 mb-3 overflow-hidden">
                                                <h2 class="font-orbitron text-sm font-bold text-white leading-6 line-clamp-2">
                                                    { &p.name }
                                                </h2>
                                            </div>

                                            // Stock
                                            <div class="mb-4">
                                                if p.in_stock {
                                                    <span class="badge-stock text-xs">
                                                        { format!("{} in stock", p.stock_count) }
                                                    </span>
                                                } else {
                                                    <span class="badge-pre text-xs">{"Out of stock"}</span>
                                                }
                                            </div>

                                            // CTA
                                            if p.in_stock {
                                                <button
                                                    onclick={add_cb}
                                                    class={if *is_current {
                                                        "w-full btn-primary text-xs py-2 mb-5"
                                                    } else {
                                                        "w-full btn-ghost text-xs py-2 mb-5"
                                                    }}
                                                >
                                                    {"Add to cart"}
                                                </button>
                                            } else {
                                                <button
                                                    disabled=true
                                                    class="w-full py-2 rounded-xl font-exo text-xs bg-navy3 text-dim border border-border cursor-not-allowed opacity-50 mb-5"
                                                >
                                                    {"Out of stock"}
                                                </button>
                                            }

                                            // Divider
                                            <div class="border-t border-border mb-5" />

                                            // Specs — price first
                                            <div class="flex flex-col gap-5">
                                                <div>
                                                    <p class="font-exo text-xs text-muted mb-1">{"Price (USD)"}</p>
                                                    <p class="font-orbitron text-lg font-bold text-orange">
                                                        { format_price(p.price) }
                                                    </p>
                                                </div>

                                                { for all_keys.iter().map(|key| {
                                                    let label = format_label(key);
                                                    let (max_val, min_val) = get_num_range(key);

                                                    let raw_value = p.attributes.as_ref()
                                                        .and_then(|a| a.get(key))
                                                        .map(|v| match v {
                                                            serde_json::Value::String(s) => s.clone(),
                                                            serde_json::Value::Number(n) => n.to_string(),
                                                            serde_json::Value::Bool(b)   => if *b { "Yes".to_string() } else { "No".to_string() },
                                                            other => other.to_string(),
                                                        })
                                                        .unwrap_or_else(|| "—".to_string());

                                                    let num_val = p.attributes.as_ref()
                                                        .and_then(|a| a.get(key))
                                                        .and_then(|v| v.as_f64());

                                                    let value_class = if let (Some(nv), Some(mx), Some(mn)) = (num_val, max_val, min_val) {
                                                        if (nv - mx).abs() < f64::EPSILON {
                                                            "font-orbitron text-lg font-bold text-green-400"
                                                        } else if (nv - mn).abs() < f64::EPSILON {
                                                            "font-orbitron text-lg font-bold text-red-400"
                                                        } else {
                                                            "font-orbitron text-lg font-bold text-white"
                                                        }
                                                    } else {
                                                        "font-orbitron text-lg font-bold text-white"
                                                    };

                                                    html! {
                                                        <div>
                                                            <p class="font-exo text-xs text-muted mb-1">{ label }</p>
                                                            <p class={value_class}>{ &raw_value }</p>
                                                        </div>
                                                    }
                                                })}
                                            </div>
                                        </div>
                                    </div>
                                }
                            })}
                        </div>

                        // Dot indicators
                        if total_cols > 1 {
                            <div class="lg:hidden flex justify-center gap-2 mt-6">
                                { for (0..total_cols).map(|i| html! {
                                    <div class={if i == *active_index {
                                        "w-2 h-2 rounded-full bg-orange transition-all duration-200"
                                    } else {
                                        "w-2 h-2 rounded-full bg-border transition-all duration-200"
                                    }} />
                                })}
                            </div>
                        }
                    </div>
                }

                <div class="h-16" />
            </div>
        </div>
    }
}

fn format_label(key: &str) -> String {
    let (name_part, unit) = if let Some(pos) = key.rfind('_') {
        let suffix = &key[pos + 1..];
        let unit = match suffix {
            "kg"  => Some("kg"),  "kn"  => Some("kN"),  "nm"  => Some("Nm"),
            "nm2" => Some("Nm²"), "s"   => Some("s"),   "m"   => Some("m"),
            "m2"  => Some("m²"),  "m3"  => Some("m³"),  "mm"  => Some("mm"),
            "bar" => Some("bar"), "w"   => Some("W"),   "hz"  => Some("Hz"),
            "ghz" => Some("GHz"), "gb"  => Some("GB"),  "km"  => Some("km"),
            "ms"  => Some("m/s"), "deg" => Some("°"),   "l"   => Some("L"),
            "mhz" => Some("MHz"), "c"   => Some("°C"),
            _     => None,
        };
        if unit.is_some() { (&key[..pos], unit) } else { (key, None) }
    } else {
        (key, None)
    };

    let label = name_part
        .replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None    => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    if let Some(u) = unit { format!("{} ({})", label, u) } else { label }
}

fn format_price(price: f64) -> String {
    if price >= 1_000_000.0 { format!("${:.1}M", price / 1_000_000.0) }
    else if price >= 1_000.0 { format!("${:.0}K", price / 1_000.0) }
    else { format!("${:.0}", price) }
}