use crate::components::layout::chatbot_widget::ChatbotWidget;
use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::components::product::product_card::ProductCard;
use crate::components::ui::spinner::{Spinner, SpinnerSize};
use crate::services::products::ProductService;
use crate::types::{ProductListItem, ProductFilters};
use crate::route::Route;

// ─── Component ───────────────────────────────────────────────────

#[function_component(Landing)]
pub fn landing() -> Html {
    let featured        = use_state(|| Vec::<ProductListItem>::new());
    let propulsion      = use_state(|| Vec::<ProductListItem>::new());
    let structural      = use_state(|| Vec::<ProductListItem>::new());
    let guidance        = use_state(|| Vec::<ProductListItem>::new());
    let payload         = use_state(|| Vec::<ProductListItem>::new());
    let loading_featured = use_state(|| true);
    let loading_groups   = use_state(|| true);
    let error            = use_state(|| Option::<String>::None);

    // Fetch featured products and all category rows on mount
    {
        let featured         = featured.clone();
        let propulsion       = propulsion.clone();
        let structural       = structural.clone();
        let guidance         = guidance.clone();
        let payload          = payload.clone();
        let loading_featured = loading_featured.clone();
        let loading_groups   = loading_groups.clone();
        let error            = error.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                // Featured — just grab the first 4 products across all groups
                let mut featured_filters = ProductFilters::new();
                featured_filters.limit = 4;

                match ProductService::list(&featured_filters).await {
                    Ok(resp) => featured.set(resp.data),
                    Err(e)   => error.set(Some(e)),
                }
                loading_featured.set(false);

                // Fetch each category — 4 products each
                let groups = [
                    ("propulsion", propulsion.clone()),
                    ("structural", structural.clone()),
                    ("guidance",   guidance.clone()),
                    ("payload",    payload.clone()),
                ];

                for (group, state) in groups {
                    let mut f = ProductFilters::new();
                    f.group = Some(group.to_string());
                    f.limit = 4;

                    match ProductService::list(&f).await {
                        Ok(resp) => state.set(resp.data),
                        Err(e)   => error.set(Some(e)),
                    }
                }

                loading_groups.set(false);
            });

            || ()
        });
    }

    html! {
        <div class="min-h-screen bg-navy">

            // ── Hero ─────────────────────────────────────────────
            <section class="relative px-6 py-24 text-center overflow-hidden">

                // Background grid decoration
                <div class="absolute inset-0 opacity-5" style="
                    background-image: linear-gradient(#f4681a 1px, transparent 1px),
                                      linear-gradient(90deg, #f4681a 1px, transparent 1px);
                    background-size: 60px 60px;
                "></div>

                <div class="relative max-w-4xl mx-auto animate-fade-up">

                    <h1 class="font-orbitron text-5xl font-bold text-white mb-6 leading-tight">
                        {"Build Your "}
                        <span class="text-orange">{"Rocket."}</span>
                        <br />
                        {"Launch Your "}
                        <span class="text-orange">{"Vision."}
                        </span>
                    </h1>

                    <p class="font-exo text-muted text-lg max-w-2xl mx-auto mb-10 leading-relaxed">
                        {"Browse our astronautics-grade components — from liquid rocket engines
                          to avionics suites. Built for engineers, researchers, and visionaries."}
                    </p>

                    <div class="flex items-center justify-center gap-4 flex-wrap">
                        <Link<Route> to={Route::Catalog}>
                            <button class="btn-primary px-8 py-3 text-base animate-pulse-glow">
                                {"Browse Catalog"}
                            </button>
                        </Link<Route>>
                        <Link<Route> to={Route::Register}>
                            <button class="btn-ghost px-8 py-3 text-base">
                                {"Create Account"}
                            </button>
                        </Link<Route>>
                    </div>
                </div>
            </section>

            // ── Error banner ─────────────────────────────────────
            if let Some(err) = (*error).clone() {
                <div class="max-w-6xl mx-auto px-6 mb-8">
                    <div class="bg-red-500/10 border border-red-500/25 rounded-xl px-4 py-3">
                        <p class="text-red-400 font-exo text-sm">
                            { format!("Failed to load products: {}", err) }
                        </p>
                    </div>
                </div>
            }

            // ── Featured components ───────────────────────────────
            <section class="px-6 pb-16">
                <div class="max-w-6xl mx-auto">
                    <div class="flex items-center justify-between mb-6">
                        <div>
                            <p class="label-mono mb-1">{"Handpicked"}</p>
                            <h2 class="font-orbitron text-xl font-bold text-white">
                                {"Featured Components"}
                            </h2>
                        </div>
                        <Link<Route> to={Route::Catalog}>
                            <span class="font-exo text-sm text-muted hover:text-orange transition-colors cursor-pointer">
                                {"View all →"}
                            </span>
                        </Link<Route>>
                    </div>

                    if *loading_featured {
                        <div class="flex justify-center py-16">
                            <Spinner size={SpinnerSize::Lg} />
                        </div>
                    } else if featured.is_empty() {
                        <p class="text-muted font-exo text-sm text-center py-8">
                            {"No featured products available."}
                        </p>
                    } else {
                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-5">
                            { for featured.iter().map(|p| html! {
                                <ProductCard product={p.clone()} />
                            })}
                        </div>
                    }
                </div>
            </section>

            // ── Category rows ─────────────────────────────────────
            if !*loading_groups {
                <>
                    { category_row("Propulsion", &propulsion) }
                    { category_row("Structural", &structural) }
                    { category_row("Guidance",    &guidance) }
                    { category_row("Payload",       &payload) }
                </>
            } else {
                <div class="flex justify-center py-16">
                    <Spinner size={SpinnerSize::Lg} />
                </div>
            }

            // ── Footer spacer ─────────────────────────────────────
            <div class="h-24" />

        </div>
    }
}

// ─── Category row helper ──────────────────────────────────────────

fn category_row(label: &str, products: &[ProductListItem]) -> Html {
    if products.is_empty() {
        return html! {};
    }

    html! {
        <section class="px-6 pb-16">
            <div class="max-w-6xl mx-auto">
                <div class="flex items-center justify-between mb-6">
                    <div>
                        <h2 class="font-orbitron text-xl font-bold text-white">{ label }</h2>
                    </div>
                    <Link<Route> to={Route::Catalog}>
                        <span class="font-exo text-sm text-muted hover:text-orange transition-colors cursor-pointer">
                            {"View all →"}
                        </span>
                    </Link<Route>>
                </div>
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-5">
                    { for products.iter().map(|p| html! {
                        <ProductCard product={p.clone()} />
                    })}
                </div>
            <ChatbotWidget />
            </div>
        </section>
    }
}