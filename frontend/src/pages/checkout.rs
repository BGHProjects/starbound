use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use crate::components::ui::tooltip::Tooltip;
use crate::context::auth::AuthContext;
use crate::context::cart::{CartContext, CartAction};
use crate::services::orders::OrderService;
use crate::types::{CreateOrderRequest, CreateOrderItem, ShippingAddress};
use crate::route::Route;

#[function_component(Checkout)]
pub fn checkout() -> Html {
    let auth      = use_context::<AuthContext>().expect("AuthContext not found");
    let cart      = use_context::<CartContext>().expect("CartContext not found");
    let navigator = use_navigator().unwrap();

    if !auth.is_authenticated() {
        navigator.push(&Route::Login);
        return html! {};
    }

    if cart.items.is_empty() {
        navigator.push(&Route::Cart);
        return html! {};
    }

    let facility_name  = use_state(|| String::new());
    let site_code      = use_state(|| String::new());
    let address_line_1 = use_state(|| String::new());
    let address_line_2 = use_state(|| String::new());
    let city           = use_state(|| String::new());
    let country        = use_state(|| String::new());
    let postal_code    = use_state(|| String::new());
    let notes          = use_state(|| String::new());
    let payment_method = use_state(|| "card".to_string());
    let loading        = use_state(|| false);
    let error          = use_state(|| Option::<String>::None);

    let subtotal = cart.total();
    let shipping = 2500.0_f64;
    let total    = subtotal + shipping;

    macro_rules! on_input {
        ($state:ident) => {{
            let $state = $state.clone();
            Callback::from(move |e: InputEvent| {
                let input = e.target_unchecked_into::<HtmlInputElement>();
                $state.set(input.value());
            })
        }};
    }

    let on_facility_name  = on_input!(facility_name);
    let on_site_code      = on_input!(site_code);
    let on_address_line_1 = on_input!(address_line_1);
    let on_address_line_2 = on_input!(address_line_2);
    let on_city           = on_input!(city);
    let on_country        = on_input!(country);
    let on_postal_code    = on_input!(postal_code);
    let on_notes          = on_input!(notes);

    let on_payment = {
        let payment_method = payment_method.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            payment_method.set(input.value());
        })
    };

    let on_submit = {
        let auth           = auth.clone();
        let cart           = cart.clone();
        let navigator      = navigator.clone();
        let loading        = loading.clone();
        let error          = error.clone();
        let facility_name  = facility_name.clone();
        let site_code      = site_code.clone();
        let address_line_1 = address_line_1.clone();
        let address_line_2 = address_line_2.clone();
        let city           = city.clone();
        let country        = country.clone();
        let postal_code    = postal_code.clone();
        let notes          = notes.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if address_line_1.is_empty() || city.is_empty()
                || country.is_empty() || postal_code.is_empty()
            {
                error.set(Some("Please fill in all required fields.".to_string()));
                return;
            }

            let token = match auth.token.clone() {
                Some(t) => t,
                None    => { navigator.push(&Route::Login); return; }
            };

            let items: Vec<CreateOrderItem> = cart.items.iter().map(|i| CreateOrderItem {
                product_id: i.product.id.clone(),
                quantity:   i.quantity,
            }).collect();

            let req = CreateOrderRequest {
                items,
                shipping_address: ShippingAddress {
                    facility_name:  (*facility_name).clone(),
                    site_code:      (*site_code).clone(),
                    address_line_1: (*address_line_1).clone(),
                    address_line_2: if address_line_2.is_empty() {
                        None
                    } else {
                        Some((*address_line_2).clone())
                    },
                    city:        (*city).clone(),
                    country:     (*country).clone(),
                    postal_code: (*postal_code).clone(),
                },
                notes: if notes.is_empty() { None } else { Some((*notes).clone()) },
            };

            loading.set(true);
            error.set(None);

            let loading   = loading.clone();
            let error     = error.clone();
            let cart      = cart.clone();
            let navigator = navigator.clone();

            spawn_local(async move {
                match OrderService::create(&req, &token).await {
                    Ok(order) => {
                        cart.dispatch(CartAction::Clear);
                        navigator.push(&Route::OrderConfirmation { id: order.id });
                    }
                    Err(e) => {
                        let msg = if e.contains("out of stock") {
                            "One or more items in your cart are out of stock.".to_string()
                        } else if e.contains("422") {
                            "Could not process order. Please check your items.".to_string()
                        } else {
                            "Something went wrong. Please try again.".to_string()
                        };
                        error.set(Some(msg));
                        loading.set(false);
                    }
                }
            });
        })
    };

    let facility_tooltip = html! {
        <Tooltip
            text="The name of your launch facility, research centre or organisation. Leave blank if not applicable."
            link="https://www.faa.gov/space/additional_information/launch_site_operator_information"
            link_label="FAA launch site info →"
        />
    };

    let site_code_tooltip = html! {
        <Tooltip
            text="ICAO or FAA site identifier for your launch complex, e.g. LC-39A. Leave blank if your facility does not have a formal designation."
            link="https://www.faa.gov/space/additional_information/launch_site_operator_information"
            link_label="FAA launch site info →"
        />
    };

    html! {
        <div class="min-h-screen bg-navy">
            <div class="max-w-6xl mx-auto px-4 py-10">

                <div class="mb-8 animate-fade-in">
                    <p class="label-mono mb-1">{"Place your order"}</p>
                    <h1 class="font-orbitron text-2xl font-bold text-white">{"Checkout"}</h1>
                </div>

                <form onsubmit={on_submit}>
                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8 animate-fade-up">

                        <div class="lg:col-span-2 space-y-8">

                            if let Some(err) = (*error).clone() {
                                <div class="bg-red-500/10 border border-red-500/25 rounded-xl px-4 py-3 animate-fade-in">
                                    <p class="text-red-400 font-exo text-sm">{ err }</p>
                                </div>
                            }

                            // ── Shipping address ──────────────────
                            <div class="card-static p-6">
                                <p class="label-mono mb-5">{"Shipping address"}</p>
                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">

                                    <div class="sm:col-span-2">
                                        <label class="label-mono text-xs mb-2 flex items-center">
                                            {"Facility name"}
                                            { facility_tooltip }
                                            <span class="text-dim text-xs font-exo normal-case tracking-normal font-normal ml-1">{"(optional)"}</span>
                                        </label>
                                        <input
                                            type="text"
                                            class="input-field"
                                            placeholder="e.g. Kennedy Space Center"
                                            value={(*facility_name).clone()}
                                            oninput={on_facility_name}
                                            disabled={*loading}
                                        />
                                    </div>

                                    <div>
                                        <label class="label-mono text-xs mb-2 flex items-center">
                                            {"Site code"}
                                            { site_code_tooltip }
                                            <span class="text-dim text-xs font-exo normal-case tracking-normal font-normal ml-1">{"(optional)"}</span>
                                        </label>
                                        <input
                                            type="text"
                                            class="input-field"
                                            placeholder="e.g. LC-39A"
                                            value={(*site_code).clone()}
                                            oninput={on_site_code}
                                            disabled={*loading}
                                        />
                                    </div>

                                    <div>
                                        <label class="label-mono text-xs mb-2 block">
                                            {"Country "}
                                            <span class="text-orange">{"*"}</span>
                                        </label>
                                        <input
                                            type="text"
                                            class="input-field"
                                            placeholder="e.g. US"
                                            value={(*country).clone()}
                                            oninput={on_country}
                                            disabled={*loading}
                                        />
                                    </div>

                                    <div class="sm:col-span-2">
                                        <label class="label-mono text-xs mb-2 block">
                                            {"Address line 1 "}
                                            <span class="text-orange">{"*"}</span>
                                        </label>
                                        <input
                                            type="text"
                                            class="input-field"
                                            placeholder="Street address"
                                            value={(*address_line_1).clone()}
                                            oninput={on_address_line_1}
                                            disabled={*loading}
                                        />
                                    </div>

                                    <div class="sm:col-span-2">
                                        <label class="label-mono text-xs mb-2 block">
                                            {"Address line 2 "}
                                            <span class="text-dim text-xs font-exo normal-case tracking-normal">{"(optional)"}</span>
                                        </label>
                                        <input
                                            type="text"
                                            class="input-field"
                                            placeholder="Suite, building, complex"
                                            value={(*address_line_2).clone()}
                                            oninput={on_address_line_2}
                                            disabled={*loading}
                                        />
                                    </div>

                                    <div>
                                        <label class="label-mono text-xs mb-2 block">
                                            {"City "}
                                            <span class="text-orange">{"*"}</span>
                                        </label>
                                        <input
                                            type="text"
                                            class="input-field"
                                            placeholder="City"
                                            value={(*city).clone()}
                                            oninput={on_city}
                                            disabled={*loading}
                                        />
                                    </div>

                                    <div>
                                        <label class="label-mono text-xs mb-2 block">
                                            {"Postal code "}
                                            <span class="text-orange">{"*"}</span>
                                        </label>
                                        <input
                                            type="text"
                                            class="input-field"
                                            placeholder="Postal / ZIP code"
                                            value={(*postal_code).clone()}
                                            oninput={on_postal_code}
                                            disabled={*loading}
                                        />
                                    </div>
                                </div>
                            </div>

                            // ── Payment method ────────────────────
                            <div class="card-static p-6">
                                <p class="label-mono mb-5">{"Payment method"}</p>
                                <div class="space-y-3">
                                    { for [
                                        ("card",   "Credit / Debit card",  "Visa, Mastercard, Amex"),
                                        ("wire",   "Bank wire transfer",   "For large orders"),
                                        ("crypto", "Cryptocurrency",       "BTC, ETH accepted"),
                                    ].iter().map(|(val, label, sub)| {
                                        let is_selected = (*payment_method) == *val;
                                        let val_str     = val.to_string();
                                        html! {
                                            <label class={if is_selected {
                                                "flex items-center gap-4 p-4 rounded-xl border-2 border-orange bg-orange/5 cursor-pointer transition-all duration-150"
                                            } else {
                                                "flex items-center gap-4 p-4 rounded-xl border border-border hover:border-orange/50 cursor-pointer transition-all duration-150"
                                            }}>
                                                <input
                                                    type="radio"
                                                    name="payment"
                                                    value={val_str}
                                                    checked={is_selected}
                                                    oninput={on_payment.clone()}
                                                    class="accent-orange"
                                                />
                                                <div>
                                                    <p class="font-exo text-sm font-semibold text-white">{ label }</p>
                                                    <p class="font-exo text-xs text-muted">{ sub }</p>
                                                </div>
                                            </label>
                                        }
                                    })}
                                </div>
                                <p class="font-exo text-xs text-dim mt-4 italic">
                                    {"Note: This is a portfolio project. No real payment will be processed."}
                                </p>
                            </div>

                            // ── Notes ─────────────────────────────
                            <div class="card-static p-6">
                                <p class="label-mono mb-5">
                                    {"Order notes "}
                                    <span class="text-dim text-xs font-exo normal-case tracking-normal font-normal">
                                        {"(optional)"}
                                    </span>
                                </p>
                                <textarea
                                    class="input-field resize-none h-24"
                                    placeholder="Special handling instructions, delivery notes..."
                                    value={(*notes).clone()}
                                    oninput={on_notes}
                                    disabled={*loading}
                                />
                            </div>
                        </div>

                        // ── Order summary ─────────────────────────
                        <div class="lg:col-span-1">
                            <div class="card-static p-6 sticky top-24">
                                <p class="label-mono mb-5">{"Order summary"}</p>

                                <div class="space-y-3 mb-5 max-h-64 overflow-y-auto">
                                    { for cart.items.iter().map(|item| html! {
                                        <div class="flex items-start justify-between gap-3">
                                            <div class="flex-1 min-w-0">
                                                <p class="font-exo text-xs text-white leading-snug truncate">
                                                    { &item.product.name }
                                                </p>
                                                <p class="font-exo text-xs text-muted">
                                                    { format!("× {}", item.quantity) }
                                                </p>
                                            </div>
                                            <span class="font-orbitron text-xs text-white flex-shrink-0">
                                                { format_price(item.line_total()) }
                                            </span>
                                        </div>
                                    })}
                                </div>

                                <div class="border-t border-border pt-4 space-y-3 mb-6">
                                    <div class="flex justify-between">
                                        <span class="font-exo text-sm text-muted">{"Subtotal"}</span>
                                        <span class="font-orbitron text-sm text-white">{ format_price(subtotal) }</span>
                                    </div>
                                    <div class="flex justify-between">
                                        <span class="font-exo text-sm text-muted">{"Shipping"}</span>
                                        <span class="font-orbitron text-sm text-white">{ format_price(shipping) }</span>
                                    </div>
                                    <div class="flex justify-between pt-3 border-t border-border">
                                        <span class="font-exo text-base font-semibold text-white">{"Total"}</span>
                                        <span class="font-orbitron text-lg font-bold text-orange">{ format_price(total) }</span>
                                    </div>
                                </div>

                                <button
                                    type="submit"
                                    class="btn-primary w-full py-3 text-sm flex items-center justify-center gap-2"
                                    disabled={*loading}
                                >
                                    if *loading {
                                        <span class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                                        {"Placing order..."}
                                    } else {
                                        {"Place order"}
                                    }
                                </button>

                                <Link<Route> to={Route::Cart}>
                                    <button type="button" class="btn-ghost w-full py-2.5 text-sm mt-3">
                                        {"← Back to cart"}
                                    </button>
                                </Link<Route>>
                            </div>
                        </div>
                    </div>
                </form>
            </div>
        </div>
    }
}

fn format_price(price: f64) -> String {
    if price >= 1_000_000.0 { format!("${:.1}M", price / 1_000_000.0) }
    else if price >= 1_000.0 { format!("${:.0}K", price / 1_000.0) }
    else { format!("${:.0}", price) }
}