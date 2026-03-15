use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::context::auth::{AuthContext, AuthAction};
use crate::services::auth::AuthService;
use crate::types::RegisterRequest;
use crate::route::Route;

#[function_component(Register)]
pub fn register() -> Html {
    let auth      = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator  = use_navigator().unwrap();

    let name     = use_state(|| String::new());
    let email    = use_state(|| String::new());
    let password = use_state(|| String::new());
    let error    = use_state(|| Option::<String>::None);
    let loading  = use_state(|| false);

    // Redirect if already logged in
    if auth.is_authenticated() {
        navigator.push(&Route::Landing);
        return html! {};
    }

    let on_name_input = {
        let name = name.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            name.set(input.value());
        })
    };

    let on_email_input = {
        let email = email.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            email.set(input.value());
        })
    };

    let on_password_input = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            password.set(input.value());
        })
    };

    let on_submit = {
        let name     = name.clone();
        let email    = email.clone();
        let password = password.clone();
        let error    = error.clone();
        let loading  = loading.clone();
        let auth     = auth.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let name_val     = (*name).clone();
            let email_val    = (*email).clone();
            let password_val = (*password).clone();
            let error        = error.clone();
            let loading      = loading.clone();
            let auth         = auth.clone();
            let navigator    = navigator.clone();

            // Client-side validation
            if name_val.is_empty() || email_val.is_empty() || password_val.is_empty() {
                error.set(Some("Please fill in all fields.".to_string()));
                return;
            }
            if name_val.len() < 2 {
                error.set(Some("Name must be at least 2 characters.".to_string()));
                return;
            }
            if password_val.len() < 8 {
                error.set(Some("Password must be at least 8 characters.".to_string()));
                return;
            }

            loading.set(true);
            error.set(None);

            spawn_local(async move {
                let req = RegisterRequest {
                    name:     name_val,
                    email:    email_val,
                    password: password_val,
                };

                match AuthService::register(req).await {
                    Ok(resp) => {
                        auth.dispatch(AuthAction::Login(resp));
                        navigator.push(&Route::Landing);
                    }
                    Err(e) => {
                        let msg = if e.contains("409") {
                            "An account with that email already exists.".to_string()
                        } else if e.contains("400") {
                            "Please check your details and try again.".to_string()
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

    html! {
        <div class="min-h-screen bg-navy flex items-center justify-center px-4">
            <div class="w-full max-w-md animate-fade-up">

                // Logo
                <div class="text-center mb-10">
                    <div class="inline-flex items-center gap-3 mb-6">
                        <div class="w-10 h-10 bg-orange rounded-xl flex items-center justify-center">
                            <span class="text-white font-orbitron font-bold text-sm">{"S"}</span>
                        </div>
                        <span class="font-orbitron text-xl font-bold text-white tracking-widest">
                            {"STAR"}
                            <span class="text-orange">{"BOUND"}</span>
                        </span>
                    </div>
                    <h1 class="font-orbitron text-2xl font-bold text-white mb-2">
                        {"Create your account"}
                    </h1>
                    <p class="text-muted font-exo text-sm">
                        {"Join Starbound and start building"}
                    </p>
                </div>

                // Card
                <div class="card-static p-8">

                    // Error message
                    if let Some(err) = (*error).clone() {
                        <div class="bg-red-500/10 border border-red-500/25 rounded-xl px-4 py-3 mb-6 animate-fade-in">
                            <p class="text-red-400 font-exo text-sm">{ err }</p>
                        </div>
                    }

                    <form onsubmit={on_submit}>

                        // Name
                        <div class="mb-5">
                            <label class="label-mono mb-2 block">{"Full name"}</label>
                            <input
                                type="text"
                                class="input-field"
                                placeholder="Ada Lovelace"
                                value={(*name).clone()}
                                oninput={on_name_input}
                                disabled={*loading}
                            />
                        </div>

                        // Email
                        <div class="mb-5">
                            <label class="label-mono mb-2 block">{"Email"}</label>
                            <input
                                type="email"
                                class="input-field"
                                placeholder="you@example.com"
                                value={(*email).clone()}
                                oninput={on_email_input}
                                disabled={*loading}
                            />
                        </div>

                        // Password
                        <div class="mb-2">
                            <label class="label-mono mb-2 block">{"Password"}</label>
                            <input
                                type="password"
                                class="input-field"
                                placeholder="Min. 8 characters"
                                value={(*password).clone()}
                                oninput={on_password_input}
                                disabled={*loading}
                            />
                        </div>

                        // Password hint
                        <p class="text-dim font-exo text-xs mb-6">
                            {"Must be at least 8 characters long"}
                        </p>

                        // Submit
                        <button
                            type="submit"
                            class="btn-primary w-full justify-center flex items-center gap-2"
                            disabled={*loading}
                        >
                            if *loading {
                                <span class="inline-block w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                                {"Creating account..."}
                            } else {
                                {"Create account"}
                            }
                        </button>

                    </form>
                </div>

                // Login link
                <p class="text-center text-muted font-exo text-sm mt-6">
                    {"Already have an account? "}
                    <Link<Route> to={Route::Login}>
                        <span class="text-orange hover:text-orange2 transition-colors cursor-pointer font-medium">
                            {"Sign in"}
                        </span>
                    </Link<Route>>
                </p>

            </div>
        </div>
    }
}