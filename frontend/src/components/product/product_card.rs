use yew::prelude::*;
use yew_router::prelude::*;
use crate::types::ProductListItem;
use crate::context::cart::{CartContext, CartAction};
use crate::route::Route;

#[derive(Properties, PartialEq)]
pub struct ProductCardProps {
    pub product: ProductListItem,
}

#[function_component(ProductCard)]
pub fn product_card(props: &ProductCardProps) -> Html {
    let cart    = use_context::<CartContext>().expect("CartContext not found");
    let product = props.product.clone();
    let added   = use_state(|| false);

    let stock_badge = if !product.in_stock {
        html! { <span class="badge-pre">{"Out of stock"}</span> }
    } else if product.stock_count <= 3 {
        html! { <span class="badge-low">{ format!("{} left", product.stock_count) }</span> }
    } else {
        html! { <span class="badge-stock">{"In stock"}</span> }
    };

    let on_add = {
        let cart    = cart.clone();
        let product = product.clone();
        let added   = added.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            cart.dispatch(CartAction::AddItem(product.clone()));
            added.set(true);
            let added = added.clone();
            gloo_timers::callback::Timeout::new(1500, move || {
                added.set(false);
            }).forget();
        })
    };

    let category_label = match product.group.as_str() {
        "structural" => "Structural",
        "guidance"   => "Guidance",
        "payload"    => "Payload",
        "propulsion" => "Propulsion",
        other        => other,
    };

    html! {
        <Link<Route> to={Route::ProductDetail { id: product.id.clone() }}>
            <div class="card group flex flex-col overflow-hidden h-full cursor-pointer">

                // Image area
                <div class="h-44 bg-navy3 border-b border-border flex items-center justify-center relative overflow-hidden transition-colors duration-200 group-hover:bg-navy4">
                    <span class="font-orbitron text-2xl font-bold text-border group-hover:text-orange transition-colors duration-200 tracking-widest select-none">
                        { &product.product_type.to_uppercase()[..4.min(product.product_type.len())] }
                    </span>
                    <div class="absolute top-3 right-3">{ stock_badge }</div>
                    <div class="absolute top-3 left-3">
                        <span class="label-mono text-xs bg-navy2 px-2 py-1 rounded-lg border border-border">
                            { category_label }
                        </span>
                    </div>
                </div>

                // Body
                <div class="p-4 flex flex-col flex-1">
                    <p class="label-mono text-xs mb-1">{ &product.product_type }</p>
                    <h3 class="font-exo font-semibold text-white text-sm leading-snug group-hover:text-orange transition-colors duration-200 mb-3">
                        { &product.name }
                    </h3>

                    <div class="flex-1" />

                    <div class="flex items-center justify-between mt-3 pt-3 border-t border-border">
                        <span class="price-text text-base">
                            { format_price(product.price) }
                        </span>
                        <button
                            onclick={on_add}
                            disabled={!product.in_stock}
                            class={if *added {
                                "px-3 py-1.5 rounded-lg text-xs font-semibold font-exo bg-green-500/20 text-green-400 border border-green-500/30 transition-all duration-200"
                            } else if product.in_stock {
                                "px-3 py-1.5 rounded-lg text-xs font-semibold font-exo bg-navy3 text-muted border border-border hover:border-orange hover:text-orange transition-all duration-200"
                            } else {
                                "px-3 py-1.5 rounded-lg text-xs font-semibold font-exo bg-navy3 text-dim border border-border cursor-not-allowed opacity-50"
                            }}
                        >
                            if *added { {"Added ✓"} } else { {"Add to cart"} }
                        </button>
                    </div>
                </div>
            </div>
        </Link<Route>>
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