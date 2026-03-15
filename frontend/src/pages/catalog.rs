use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use gloo_timers::callback::Timeout;
use std::cell::RefCell;
use std::rc::Rc;
use crate::components::product::product_card::ProductCard;
use crate::components::ui::spinner::{Spinner, SpinnerSize};
use crate::services::products::ProductService;
use crate::types::{ProductListItem, ProductFilters};
use crate::route::Route;

#[derive(Clone, PartialEq)]
struct GroupOption {
    value: &'static str,
    label: &'static str,
    types: &'static [(&'static str, &'static str)],
}

const GROUPS: &[GroupOption] = &[
    GroupOption {
        value: "propulsion",
        label: "Propulsion",
        types: &[
            ("liquid_engine",   "Liquid Rocket Engines"),
            ("propellant_tank", "Propellant Tanks"),
            ("rocket_nozzle",   "Rocket Nozzles"),
        ],
    },
    GroupOption {
        value: "structural",
        label: "Structural",
        types: &[
            ("rocket_frame",    "Rocket Frames"),
            ("panels_fuselage", "Panels & Fuselage"),
            ("control_fins",    "Control Fins"),
        ],
    },
    GroupOption {
        value: "guidance",
        label: "Guidance",
        types: &[
            ("flight_computer",   "Flight Computers"),
            ("nav_sensors",       "Navigation Sensors"),
            ("control_actuation", "Control Actuation"),
            ("telemetry",         "Telemetry"),
        ],
    },
    GroupOption {
        value: "payload",
        label: "Payload",
        types: &[
            ("nose_cone",    "Nose Cones"),
            ("crewed_cabin", "Crewed Cabins"),
            ("cargo_module", "Cargo Modules"),
        ],
    },
];

#[derive(Clone, PartialEq)]
enum GridPhase {
    Idle,
    Exiting,
    Entering,
}

#[function_component(Catalog)]
pub fn catalog() -> Html {
    let selected_group = use_state(|| Option::<String>::None);
    let selected_type  = use_state(|| Option::<String>::None);
    let in_stock_only  = use_state(|| false);
    let min_price      = use_state(|| 0u32);
    let max_price      = use_state(|| 50_000_000u32);
    let search_term    = use_state(|| String::new());
    let current_page   = use_state(|| 1i32);

    let displayed      = use_state(|| Vec::<ProductListItem>::new());
    let total          = use_state(|| 0i32);
    let loading        = use_state(|| true);
    let error          = use_state(|| Option::<String>::None);
    let phase          = use_state(|| GridPhase::Idle);
    let filters_open   = use_state(|| false);

    // Committed price values — only update after debounce settles
    let committed_min  = use_state(|| 0u32);
    let committed_max  = use_state(|| 50_000_000u32);

    let min_debounce: Rc<RefCell<Option<Timeout>>> = use_mut_ref(|| None);
    let max_debounce: Rc<RefCell<Option<Timeout>>> = use_mut_ref(|| None);

    // ── Fetch when committed filters change ───────────────────────
    {
        let displayed  = displayed.clone();
        let total      = total.clone();
        let loading    = loading.clone();
        let error      = error.clone();
        let phase      = phase.clone();
        let group      = (*selected_group).clone();
        let type_      = (*selected_type).clone();
        let stock_only = *in_stock_only;
        let page       = *current_page;
        let search     = (*search_term).clone();
        let c_min      = *committed_min;
        let c_max      = *committed_max;

        use_effect_with(
            (group.clone(), type_.clone(), stock_only, page, search.clone(), c_min, c_max),
            move |_| {
                let has_current = !(*displayed).is_empty();
                if has_current {
                    phase.set(GridPhase::Exiting);
                } else {
                    loading.set(true);
                }

                let displayed = displayed.clone();
                let total     = total.clone();
                let loading   = loading.clone();
                let error     = error.clone();
                let phase     = phase.clone();
                let delay     = if has_current { 250 } else { 0 };

                Timeout::new(delay, move || {
                    spawn_local(async move {
                        let mut filters = ProductFilters::new();
                        filters.group  = group;
                        filters.type_  = type_;
                        filters.search = if search.is_empty() { None } else { Some(search) };
                        filters.page   = page;
                        filters.limit  = 12;

                        match ProductService::list(&filters).await {
                            Ok(resp) => {
                                let mut data = resp.data;
                                if stock_only {
                                    data.retain(|p| p.in_stock);
                                }
                                data.retain(|p| {
                                    let price = p.price as u32;
                                    price >= c_min && price <= c_max
                                });

                                total.set(resp.total);
                                displayed.set(data);
                                phase.set(GridPhase::Entering);
                                loading.set(false);

                                let phase = phase.clone();
                                Timeout::new(600, move || {
                                    phase.set(GridPhase::Idle);
                                }).forget();
                            }
                            Err(e) => {
                                error.set(Some(e));
                                loading.set(false);
                                phase.set(GridPhase::Idle);
                            }
                        }
                    });
                }).forget();

                || ()
            },
        );
    }

    // ── Callbacks ─────────────────────────────────────────────────

    let on_group_click = {
        let selected_group = selected_group.clone();
        let selected_type  = selected_type.clone();
        let current_page   = current_page.clone();
        Callback::from(move |group: String| {
            if (*selected_group).as_deref() == Some(&group) {
                selected_group.set(None);
            } else {
                selected_group.set(Some(group));
                selected_type.set(None);
            }
            current_page.set(1);
        })
    };

    let on_type_click = {
        let selected_type = selected_type.clone();
        let current_page  = current_page.clone();
        Callback::from(move |t: String| {
            if (*selected_type).as_deref() == Some(&t) {
                selected_type.set(None);
            } else {
                selected_type.set(Some(t));
            }
            current_page.set(1);
        })
    };

    let on_stock_toggle = {
        let in_stock_only = in_stock_only.clone();
        let current_page  = current_page.clone();
        Callback::from(move |_: MouseEvent| {
            in_stock_only.set(!*in_stock_only);
            current_page.set(1);
        })
    };

    let on_search_input = {
        let search_term  = search_term.clone();
        let current_page = current_page.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            search_term.set(input.value());
            current_page.set(1);
        })
    };

    // Price sliders — update display value immediately,
    // cancel any pending debounce, schedule a fresh one
    let on_min_price_input = {
        let min_price      = min_price.clone();
        let committed_min  = committed_min.clone();
        let current_page   = current_page.clone();
        let min_debounce = Rc::clone(&min_debounce);

        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            if let Ok(v) = input.value().parse::<u32>() {
                // Update slider display immediately — no re-render delay
                min_price.set(v);

                // Cancel the previous pending commit
                *min_debounce.borrow_mut() = None;

                // Schedule a new commit 400ms from now
                let committed_min  = committed_min.clone();
                let current_page   = current_page.clone();
                let min_debounce2 = Rc::clone(&min_debounce);

                let t = Timeout::new(400, move || {
                    committed_min.set(v);
                    current_page.set(1);
                    *min_debounce2.borrow_mut() = None;
                });

                *min_debounce.borrow_mut() = Some(t);
            }
        })
    };

    let on_max_price_input = {
        let max_price      = max_price.clone();
        let committed_max  = committed_max.clone();
        let current_page   = current_page.clone();
        let max_debounce = Rc::clone(&max_debounce);

        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            if let Ok(v) = input.value().parse::<u32>() {
                max_price.set(v);

                // Cancel the previous pending commit
                *max_debounce.borrow_mut() = None;

                let committed_max  = committed_max.clone();
                let current_page   = current_page.clone();
                let max_debounce2 = Rc::clone(&max_debounce);

                let t = Timeout::new(400, move || {
                    committed_max.set(v);
                    current_page.set(1);
                    *max_debounce2.borrow_mut() = None;
                });

                *max_debounce.borrow_mut() = Some(t);
            }
        })
    };

    let on_clear_filters = {
        let selected_group = selected_group.clone();
        let selected_type  = selected_type.clone();
        let in_stock_only  = in_stock_only.clone();
        let min_price      = min_price.clone();
        let max_price      = max_price.clone();
        let committed_min  = committed_min.clone();
        let committed_max  = committed_max.clone();
        let search_term    = search_term.clone();
        let current_page   = current_page.clone();
        let min_debounce = Rc::clone(&min_debounce);
        let max_debounce = Rc::clone(&max_debounce);
        Callback::from(move |_: MouseEvent| {
            // Cancel any in-flight debounce
            *min_debounce.borrow_mut() = None;
            *max_debounce.borrow_mut() = None;
            selected_group.set(None);
            selected_type.set(None);
            in_stock_only.set(false);
            min_price.set(0);
            max_price.set(50_000_000);
            committed_min.set(0);
            committed_max.set(50_000_000);
            search_term.set(String::new());
            current_page.set(1);
        })
    };

    let toggle_filters = {
        let filters_open = filters_open.clone();
        Callback::from(move |_: MouseEvent| {
            filters_open.set(!*filters_open);
        })
    };

    // ── Derived ───────────────────────────────────────────────────

    let has_active_filters = selected_group.is_some()
        || selected_type.is_some()
        || *in_stock_only
        || *committed_min > 0
        || *committed_max < 50_000_000;

    let total_pages = ((*total as f32) / 12.0).ceil() as i32;

    let grid_class = match *phase {
        GridPhase::Exiting  =>
            "grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-5 transition-all duration-250 opacity-0 translate-y-2 pointer-events-none",
        GridPhase::Entering |
        GridPhase::Idle     =>
            "grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-5",
    };

    // ── Filter sidebar ────────────────────────────────────────────
    let filter_content = {
        let selected_group = (*selected_group).clone();
        let selected_type  = (*selected_type).clone();

        html! {
            <div class="space-y-6">
                if has_active_filters {
                    <div class="flex items-center justify-between">
                        <span class="label-mono text-orange">{"Active filters"}</span>
                        <button
                            onclick={on_clear_filters}
                            class="font-exo text-xs text-muted hover:text-orange transition-colors"
                        >
                            {"Clear all"}
                        </button>
                    </div>
                }

                <div>
                    <p class="label-mono mb-3">{"Category"}</p>
                    <div class="space-y-1">
                        { for GROUPS.iter().map(|g| {
                            let is_active = selected_group.as_deref() == Some(g.value);
                            let group_val = g.value.to_string();
                            let on_click  = on_group_click.clone();
                            html! {
                                <div>
                                    <button
                                        onclick={Callback::from(move |_| on_click.emit(group_val.clone()))}
                                        class={if is_active {
                                            "w-full text-left px-3 py-2 rounded-lg font-exo text-sm font-medium text-orange bg-orange/10 border border-orange/20 transition-all duration-150"
                                        } else {
                                            "w-full text-left px-3 py-2 rounded-lg font-exo text-sm text-muted hover:text-white hover:bg-navy3 transition-all duration-150"
                                        }}
                                    >
                                        { g.label }
                                    </button>
                                    if is_active {
                                        <div class="ml-3 mt-1 space-y-1 border-l border-border pl-3 animate-fade-in">
                                            { for g.types.iter().map(|(type_val, type_label)| {
                                                let is_type_active = selected_type.as_deref() == Some(type_val);
                                                let tv      = type_val.to_string();
                                                let on_type = on_type_click.clone();
                                                html! {
                                                    <button
                                                        onclick={Callback::from(move |_| on_type.emit(tv.clone()))}
                                                        class={if is_type_active {
                                                            "w-full text-left px-3 py-1.5 rounded-lg font-exo text-xs font-medium text-orange bg-orange/10 transition-all duration-150"
                                                        } else {
                                                            "w-full text-left px-3 py-1.5 rounded-lg font-exo text-xs text-dim hover:text-muted transition-all duration-150"
                                                        }}
                                                    >
                                                        { type_label }
                                                    </button>
                                                }
                                            })}
                                        </div>
                                    }
                                </div>
                            }
                        })}
                    </div>
                </div>

                <div>
                    <p class="label-mono mb-3">{"Availability"}</p>
                    <button
                        onclick={on_stock_toggle}
                        class="flex items-center gap-3 w-full"
                    >
                        <div class={if *in_stock_only {
                            "w-10 h-6 rounded-full bg-orange transition-colors duration-200 relative flex-shrink-0"
                        } else {
                            "w-10 h-6 rounded-full bg-navy4 border border-border transition-colors duration-200 relative flex-shrink-0"
                        }}>
                            <div class={if *in_stock_only {
                                "absolute top-1 right-1 w-4 h-4 bg-white rounded-full transition-all duration-200"
                            } else {
                                "absolute top-1 left-1 w-4 h-4 bg-muted rounded-full transition-all duration-200"
                            }}></div>
                        </div>
                        <span class="font-exo text-sm text-muted">{"In stock only"}</span>
                    </button>
                </div>

                <div>
                    <p class="label-mono mb-3">{"Price range"}</p>
                    <div class="space-y-3">
                        <div>
                            <div class="flex justify-between mb-1">
                                <span class="font-exo text-xs text-dim">{"Min"}</span>
                                <span class="font-orbitron text-xs text-orange">
                                    { format_price_short(*min_price) }
                                </span>
                            </div>
                            <input
                                type="range"
                                min="0"
                                max="50000000"
                                step="100000"
                                value={min_price.to_string()}
                                oninput={on_min_price_input}
                                class="w-full accent-orange cursor-pointer"
                            />
                        </div>
                        <div>
                            <div class="flex justify-between mb-1">
                                <span class="font-exo text-xs text-dim">{"Max"}</span>
                                <span class="font-orbitron text-xs text-orange">
                                    { format_price_short(*max_price) }
                                </span>
                            </div>
                            <input
                                type="range"
                                min="0"
                                max="50000000"
                                step="100000"
                                value={max_price.to_string()}
                                oninput={on_max_price_input}
                                class="w-full accent-orange cursor-pointer"
                            />
                        </div>
                    </div>
                </div>
            </div>
        }
    };

    let group_label = (*selected_group).as_deref()
        .and_then(|g| GROUPS.iter().find(|x| x.value == g))
        .map(|g| g.label);

    html! {
        <div class="min-h-screen bg-navy">
            <div class="max-w-7xl mx-auto px-4 py-8">

                <div class="relative mb-6">
                    <div class="absolute left-4 top-1/2 -translate-y-1/2 text-muted text-sm pointer-events-none">
                        {"⌕"}
                    </div>
                    <input
                        type="text"
                        class="input-field pl-10"
                        placeholder="Search components, SKUs..."
                        value={(*search_term).clone()}
                        oninput={on_search_input}
                    />
                </div>

                <div class="flex gap-6">

                    <aside class="hidden lg:block w-56 flex-shrink-0">
                        <div class="card-static p-5 sticky top-24">
                            { filter_content.clone() }
                        </div>
                    </aside>

                    <div class="flex-1 min-w-0">

                        <div class="flex items-center justify-between mb-6 flex-wrap gap-3">
                            <div>
                                if let Some(label) = group_label {
                                    <p class="label-mono mb-1">{ label }</p>
                                }
                                <h1 class="font-orbitron text-xl font-bold text-white">
                                    if (*search_term).is_empty() {
                                        if (*selected_group).is_none() {
                                            {"All Components"}
                                        } else {
                                            {"Results"}
                                        }
                                    } else {
                                        { format!("Results for \"{}\"", *search_term) }
                                    }
                                </h1>
                                if !*loading {
                                    <p class="font-exo text-xs text-muted mt-1">
                                        { format!("{} products found", (*displayed).len()) }
                                    </p>
                                }
                            </div>
                            <button
                                onclick={toggle_filters}
                                class="lg:hidden flex items-center gap-2 btn-ghost text-sm"
                            >
                                {"Filters"}
                                if has_active_filters {
                                    <span class="w-2 h-2 bg-orange rounded-full"></span>
                                }
                            </button>
                        </div>

                        if *filters_open {
                            <div class="lg:hidden card-static p-5 mb-6 animate-fade-in">
                                { filter_content }
                            </div>
                        }

                        if *loading {
                            <div class="flex justify-center py-24">
                                <Spinner size={SpinnerSize::Lg} />
                            </div>
                        } else if let Some(err) = (*error).clone() {
                            <div class="bg-red-500/10 border border-red-500/25 rounded-xl px-4 py-3">
                                <p class="text-red-400 font-exo text-sm">
                                    { format!("Error: {}", err) }
                                </p>
                            </div>
                        } else if (*displayed).is_empty() {
                            <div class="text-center py-24 animate-fade-in">
                                <p class="font-orbitron text-lg text-muted mb-2">
                                    {"No results found"}
                                </p>
                                <p class="font-exo text-sm text-dim">
                                    {"Try adjusting your filters or search term"}
                                </p>
                            </div>
                        } else {
                            <>
                                <div class={grid_class}>
                                    { for (*displayed).iter().enumerate().map(|(i, p)| {
                                        let delay = format!(
                                            "animation-delay: {}ms",
                                            (i * 50).min(400)
                                        );
                                        let card_class = if *phase == GridPhase::Entering {
                                            "opacity-0 animate-fade-up"
                                        } else {
                                            ""
                                        };
                                        html! {
                                            <div class={card_class} style={delay}>
                                                <ProductCard product={p.clone()} />
                                            </div>
                                        }
                                    })}
                                </div>

                                if total_pages > 1 {
                                    <div class="flex items-center justify-center gap-2 mt-10">
                                        <button
                                            onclick={{
                                                let current_page = current_page.clone();
                                                let page = *current_page;
                                                Callback::from(move |_| {
                                                    if page > 1 { current_page.set(page - 1); }
                                                })
                                            }}
                                            disabled={*current_page <= 1}
                                            class="btn-ghost text-sm px-4 py-2 disabled:opacity-40"
                                        >
                                            {"← Prev"}
                                        </button>
                                        <span class="font-orbitron text-xs text-muted px-4">
                                            { format!("{} / {}", *current_page, total_pages) }
                                        </span>
                                        <button
                                            onclick={{
                                                let current_page = current_page.clone();
                                                let page = *current_page;
                                                Callback::from(move |_| {
                                                    if page < total_pages {
                                                        current_page.set(page + 1);
                                                    }
                                                })
                                            }}
                                            disabled={*current_page >= total_pages}
                                            class="btn-ghost text-sm px-4 py-2 disabled:opacity-40"
                                        >
                                            {"Next →"}
                                        </button>
                                    </div>
                                }
                            </>
                        }
                    </div>
                </div>
            </div>
        </div>
    }
}

fn format_price_short(price: u32) -> String {
    if price >= 1_000_000 {
        format!("${:.0}M", price as f64 / 1_000_000.0)
    } else if price >= 1_000 {
        format!("${:.0}K", price as f64 / 1_000.0)
    } else {
        format!("${}", price)
    }
}