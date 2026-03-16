use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlElement, Element};
use gloo_timers::callback::Timeout;
use crate::context::chat::{ChatContext, ChatAction};
use crate::services::api::ApiClient;
use crate::types::{ChatMessage, ChatRequest, ChatResponse};
use crate::route::Route;

#[function_component(Chat)]
pub fn chat() -> Html {
    let chat_ctx   = use_context::<ChatContext>().expect("ChatContext not found");
    let input      = use_state(|| String::new());
    let scroll_ref = use_node_ref();

    // Scroll to bottom whenever messages change
    {
        let scroll_ref = scroll_ref.clone();
        let msg_count  = chat_ctx.messages.len();
        use_effect_with(msg_count, move |_| {
            Timeout::new(50, move || {
                if let Some(el) = scroll_ref.cast::<Element>() {
                    if let Ok(html_el) = el.dyn_into::<HtmlElement>() {
                        html_el.set_scroll_top(html_el.scroll_height());
                    }
                }
            }).forget();
            || ()
        });
    }

    let on_input = {
        let input = input.clone();
        Callback::from(move |e: InputEvent| {
            let el = e.target_unchecked_into::<HtmlInputElement>();
            input.set(el.value());
        })
    };

    let on_send = {
        let input    = input.clone();
        let chat_ctx = chat_ctx.clone();

        Callback::from(move |_: MouseEvent| {
            let query = (*input).trim().to_string();
            if query.is_empty() || chat_ctx.is_loading { return; }

            input.set(String::new());

            // Add user message immediately
            chat_ctx.dispatch(ChatAction::AddMessage(ChatMessage {
                role:    "user".to_string(),
                content: query.clone(),
            }));
            chat_ctx.dispatch(ChatAction::SetLoading(true));

            let chat_ctx   = chat_ctx.clone();
            let session_id = chat_ctx.session_id.clone();

            spawn_local(async move {
                let req = ChatRequest {
                    query,
                    session_id,
                };

                match ApiClient::post::<ChatRequest, ChatResponse>(
                    "/chat",
                    &req,
                    None,
                ).await {
                    Ok(resp) => {
                        chat_ctx.dispatch(ChatAction::AddMessage(ChatMessage {
                            role:    "assistant".to_string(),
                            content: resp.answer,
                        }));
                        chat_ctx.dispatch(ChatAction::SetLoading(false));
                    }
                    Err(e) => {
                        chat_ctx.dispatch(ChatAction::AddMessage(ChatMessage {
                            role:    "assistant".to_string(),
                            content: format!(
                                "Sorry, I couldn't reach the chat service right now. \
                                 Make sure the RAG service is running on port 8001. \
                                 ({})", e
                            ),
                        }));
                        chat_ctx.dispatch(ChatAction::SetLoading(false));
                    }
                }
            });
        })
    };

    let on_key_down = {
        let on_send = on_send.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" && !e.shift_key() {
                e.prevent_default();
                on_send.emit(MouseEvent::new("click").unwrap());
            }
        })
    };

    let on_clear = {
        let chat_ctx = chat_ctx.clone();
        Callback::from(move |_: MouseEvent| {
            chat_ctx.dispatch(ChatAction::Clear);
        })
    };

    let is_empty = chat_ctx.messages.is_empty();

    html! {
        <div class="flex flex-col bg-navy" style="height: calc(100vh - 64px);">

            // ── Header ────────────────────────────────────────────
            <div class="flex items-center justify-between px-6 py-4
                        border-b border-border flex-shrink-0">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 bg-orange/10 border border-orange/30
                                rounded-lg flex items-center justify-center">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none"
                             stroke="#f4681a" stroke-width="2" stroke-linecap="round">
                            <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/>
                        </svg>
                    </div>
                    <div>
                        <h1 class="font-orbitron text-sm font-bold text-white">
                            {"Starbound AI"}
                        </h1>
                        <p class="font-exo text-xs text-muted">
                            {"Ask me anything about our rocket components"}
                        </p>
                    </div>
                </div>

                <div class="flex items-center gap-3">
                    if !is_empty {
                        <button
                            onclick={on_clear}
                            class="font-exo text-xs text-dim hover:text-muted transition-colors"
                        >
                            {"Clear chat"}
                        </button>
                    }
                    <Link<Route> to={Route::Catalog}>
                        <button class="btn-ghost text-xs px-3 py-1.5">
                            {"← Catalog"}
                        </button>
                    </Link<Route>>
                </div>
            </div>

            // ── Messages ──────────────────────────────────────────
            <div
                ref={scroll_ref}
                class="flex-1 overflow-y-auto px-4 py-6 space-y-6"
            >
                if is_empty {
                    // Empty state — suggested prompts
                    <div class="max-w-2xl mx-auto animate-fade-up">
                        <div class="text-center mb-10">
                            <div class="w-16 h-16 bg-orange/10 border border-orange/20
                                        rounded-2xl flex items-center justify-center mx-auto mb-4">
                                <svg width="28" height="28" viewBox="0 0 24 24" fill="none"
                                     stroke="#f4681a" stroke-width="1.5" stroke-linecap="round">
                                    <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/>
                                </svg>
                            </div>
                            <h2 class="font-orbitron text-xl font-bold text-white mb-2">
                                {"How can I help?"}
                            </h2>
                            <p class="font-exo text-sm text-muted">
                                {"Ask me about our components, specifications, or help building your rocket"}
                            </p>
                        </div>

                        // Suggested prompts
                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                            { for [
                                ("🚀", "Build me a rocket under $1 million"),
                                ("🌡️", "What components can withstand 400°C?"),
                                ("⚡", "How can I increase my rocket's top speed?"),
                                ("🛸", "What payload options do you have?"),
                            ].iter().map(|(icon, prompt)| {
                                let chat_ctx  = chat_ctx.clone();
                                let prompt_str = prompt.to_string();
                                html! {
                                    <button
                                        onclick={Callback::from(move |_: MouseEvent| {
                                            let query = prompt_str.clone();
                                            chat_ctx.dispatch(ChatAction::AddMessage(ChatMessage {
                                                role:    "user".to_string(),
                                                content: query.clone(),
                                            }));
                                            chat_ctx.dispatch(ChatAction::SetLoading(true));

                                            let chat_ctx   = chat_ctx.clone();
                                            let session_id = chat_ctx.session_id.clone();

                                            spawn_local(async move {
                                                let req = ChatRequest {
                                                    query,
                                                    session_id,
                                                };
                                                match ApiClient::post::<ChatRequest, ChatResponse>(
                                                    "/chat", &req, None,
                                                ).await {
                                                    Ok(resp) => {
                                                        chat_ctx.dispatch(ChatAction::AddMessage(ChatMessage {
                                                            role:    "assistant".to_string(),
                                                            content: resp.answer,
                                                        }));
                                                    }
                                                    Err(e) => {
                                                        chat_ctx.dispatch(ChatAction::AddMessage(ChatMessage {
                                                            role:    "assistant".to_string(),
                                                            content: format!("Sorry, I couldn't reach the chat service. ({})", e),
                                                        }));
                                                    }
                                                }
                                                chat_ctx.dispatch(ChatAction::SetLoading(false));
                                            });
                                        })}
                                        class="card p-4 text-left hover:border-orange
                                               transition-all duration-200 group"
                                    >
                                        <span class="text-lg mr-2">{ icon }</span>
                                        <span class="font-exo text-sm text-muted
                                                     group-hover:text-white transition-colors">
                                            { prompt }
                                        </span>
                                    </button>
                                }
                            })}
                        </div>
                    </div>
                } else {
                    <div class="max-w-2xl mx-auto space-y-6">
                        { for chat_ctx.messages.iter().enumerate().map(|(i, msg)| {
                            let is_user = msg.role == "user";
                            let delay   = format!("animation-delay: {}ms", (i * 30).min(300));
                            html! {
                                <div
                                    class={if is_user {
                                        "opacity-0 animate-fade-up flex justify-end"
                                    } else {
                                        "opacity-0 animate-fade-up flex justify-start"
                                    }}
                                    style={delay}
                                >
                                    if !is_user {
                                        // Assistant avatar
                                        <div class="w-7 h-7 bg-orange/10 border border-orange/20
                                                    rounded-lg flex items-center justify-center
                                                    flex-shrink-0 mr-3 mt-1">
                                            <svg width="12" height="12" viewBox="0 0 24 24"
                                                 fill="none" stroke="#f4681a" stroke-width="2">
                                                <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/>
                                            </svg>
                                        </div>
                                    }
                                    <div class={if is_user {
                                        "max-w-xs sm:max-w-md bg-orange text-white
                                         rounded-2xl rounded-tr-sm px-4 py-3"
                                    } else {
                                        "max-w-xs sm:max-w-lg bg-navy2 border border-border
                                         text-white rounded-2xl rounded-tl-sm px-4 py-3"
                                    }}>
                                        <p class="font-exo text-sm leading-relaxed whitespace-pre-wrap">
                                            { &msg.content }
                                        </p>
                                    </div>
                                </div>
                            }
                        })}

                        // Typing indicator
                        if chat_ctx.is_loading {
                            <div class="flex justify-start animate-fade-in">
                                <div class="w-7 h-7 bg-orange/10 border border-orange/20
                                            rounded-lg flex items-center justify-center
                                            flex-shrink-0 mr-3 mt-1">
                                    <svg width="12" height="12" viewBox="0 0 24 24"
                                         fill="none" stroke="#f4681a" stroke-width="2">
                                        <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/>
                                    </svg>
                                </div>
                                <div class="bg-navy2 border border-border rounded-2xl rounded-tl-sm px-4 py-3">
                                    <div class="flex items-center gap-1.5">
                                        <div class="w-1.5 h-1.5 bg-muted rounded-full animate-bounce"
                                             style="animation-delay: 0ms" />
                                        <div class="w-1.5 h-1.5 bg-muted rounded-full animate-bounce"
                                             style="animation-delay: 150ms" />
                                        <div class="w-1.5 h-1.5 bg-muted rounded-full animate-bounce"
                                             style="animation-delay: 300ms" />
                                    </div>
                                </div>
                            </div>
                        }
                    </div>
                }
            </div>

            // ── Input bar ─────────────────────────────────────────
            <div class="flex-shrink-0 border-t border-border px-4 py-4">
                <div class="max-w-2xl mx-auto flex gap-3 items-end">
                    <div class="flex-1 relative">
                        <textarea
                            class="input-field resize-none pr-4 py-3 min-h-12 max-h-32"
                            placeholder="Ask about rocket components..."
                            value={(*input).clone()}
                            oninput={on_input}
                            onkeydown={on_key_down}
                            rows="1"
                            disabled={chat_ctx.is_loading}
                        />
                    </div>
                    <button
                        onclick={on_send}
                        disabled={(*input).trim().is_empty() || chat_ctx.is_loading}
                        class="btn-primary px-4 py-3 flex-shrink-0 flex items-center
                               justify-center disabled:opacity-40 disabled:cursor-not-allowed"
                    >
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none"
                             stroke="currentColor" stroke-width="2" stroke-linecap="round">
                            <line x1="22" y1="2" x2="11" y2="13"/>
                            <polygon points="22 2 15 22 11 13 2 9 22 2"/>
                        </svg>
                    </button>
                </div>
                <p class="text-center font-exo text-xs text-dim mt-2">
                    {"Press Enter to send · Shift+Enter for new line"}
                </p>
            </div>
        </div>
    }
}