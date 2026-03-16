use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use crate::context::chat::{ChatContext, ChatAction};
use crate::services::api::ApiClient;
use crate::types::{ChatMessage, ChatRequest, ChatResponse};
use crate::components::ui::chat_message::ChatMessageContent;
use crate::route::Route;

#[function_component(ChatbotWidget)]
pub fn chatbot_widget() -> Html {
    let chat_ctx = use_context::<ChatContext>().expect("ChatContext not found");
    let open     = use_state(|| false);
    let input    = use_state(|| String::new());

    let toggle = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(!*open))
    };

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

            chat_ctx.dispatch(ChatAction::AddMessage(ChatMessage {
                role:    "user".to_string(),
                content: query.clone(),
            }));
            chat_ctx.dispatch(ChatAction::SetLoading(true));

            let chat_ctx   = chat_ctx.clone();
            let session_id = chat_ctx.session_id.clone();

            spawn_local(async move {
                let req = ChatRequest { query, session_id };
                match ApiClient::post::<ChatRequest, ChatResponse>("/chat", &req, None).await {
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

    // Only show the last 6 messages in the widget to keep it compact
    let visible_messages: Vec<ChatMessage> = chat_ctx.messages
        .iter()
        .rev()
        .take(6)
        .rev()
        .cloned()
        .collect();

    html! {
        <div class="fixed bottom-6 right-6 z-50 flex flex-col items-end gap-3">

            // ── Chat window ───────────────────────────────────────
            if *open {
                <div class="w-80 bg-navy2 border border-border rounded-2xl
                             overflow-hidden animate-scale-in shadow-lg"
                     style="max-height: 480px; display: flex; flex-direction: column;">

                    // Header
                    <div class="flex items-center justify-between px-4 py-3
                                border-b border-border flex-shrink-0">
                        <div class="flex items-center gap-2">
                            <div class="w-6 h-6 bg-orange/10 border border-orange/20
                                        rounded-lg flex items-center justify-center">
                                <svg width="10" height="10" viewBox="0 0 24 24" fill="none"
                                     stroke="#f4681a" stroke-width="2.5">
                                    <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/>
                                </svg>
                            </div>
                            <span class="font-orbitron text-xs font-bold text-white">
                                {"Starbound AI"}
                            </span>
                        </div>
                        <div class="flex items-center gap-2">
                            // Open full chat button
                            <Link<Route> to={Route::Chat}>
                                <button
                                    class="font-exo text-xs text-muted hover:text-orange
                                           transition-colors flex items-center gap-1"
                                    title="Open full chat"
                                >
                                    <svg width="11" height="11" viewBox="0 0 24 24" fill="none"
                                         stroke="currentColor" stroke-width="2">
                                        <polyline points="15 3 21 3 21 9"/>
                                        <path d="M10 14L21 3"/>
                                        <polyline points="21 14 21 21 3 21 3 3 10 3"/>
                                    </svg>
                                    {"Expand"}
                                </button>
                            </Link<Route>>
                            <button
                                onclick={toggle.clone()}
                                class="text-dim hover:text-muted transition-colors text-lg leading-none"
                            >
                                {"×"}
                            </button>
                        </div>
                    </div>

                    // Messages
                    <div class="flex-1 overflow-y-auto px-3 py-3 space-y-3 min-h-0">
                        if visible_messages.is_empty() {
                            <div class="text-center py-6">
                                <p class="font-exo text-xs text-muted mb-3">
                                    {"Ask me about rocket components"}
                                </p>
                                <div class="space-y-2">
                                    { for [
                                        "Build a rocket under $1M",
                                        "What can withstand 400°C?",
                                        "Best engines for speed",
                                    ].iter().map(|prompt| {
                                        let chat_ctx  = chat_ctx.clone();
                                        let query_str = prompt.to_string();
                                        html! {
                                            <button
                                                onclick={Callback::from(move |_: MouseEvent| {
                                                    let query = query_str.clone();
                                                    chat_ctx.dispatch(ChatAction::AddMessage(ChatMessage {
                                                        role:    "user".to_string(),
                                                        content: query.clone(),
                                                    }));
                                                    chat_ctx.dispatch(ChatAction::SetLoading(true));
                                                    let chat_ctx   = chat_ctx.clone();
                                                    let session_id = chat_ctx.session_id.clone();
                                                    spawn_local(async move {
                                                        let req = ChatRequest { query, session_id };
                                                        match ApiClient::post::<ChatRequest, ChatResponse>("/chat", &req, None).await {
                                                            Ok(r) => chat_ctx.dispatch(ChatAction::AddMessage(ChatMessage {
                                                                role: "assistant".to_string(), content: r.answer,
                                                            })),
                                                            Err(e) => chat_ctx.dispatch(ChatAction::AddMessage(ChatMessage {
                                                                role: "assistant".to_string(),
                                                                content: format!("Sorry, chat service unavailable. ({})", e),
                                                            })),
                                                        }
                                                        chat_ctx.dispatch(ChatAction::SetLoading(false));
                                                    });
                                                })}
                                                class="w-full text-left px-3 py-2 rounded-lg
                                                       bg-navy3 border border-border
                                                       font-exo text-xs text-muted
                                                       hover:border-orange hover:text-white
                                                       transition-all duration-150"
                                            >
                                                { prompt }
                                            </button>
                                        }
                                    })}
                                </div>
                            </div>
                        } else {
                            { for visible_messages.iter().map(|msg| {
                                let is_user = msg.role == "user";
                                html! {
                                    <div class={if is_user {
                                        "flex justify-end"
                                    } else {
                                        "flex justify-start"
                                    }}>
                                        <div class={if is_user {
                                            "max-w-48 bg-orange text-white rounded-xl
                                             rounded-tr-sm px-3 py-2"
                                        } else {
                                            "max-w-56 bg-navy3 border border-border
                                             text-white rounded-xl rounded-tl-sm px-3 py-2"
                                        }}>
                                            <p class="font-exo text-xs leading-relaxed">
                                                <ChatMessageContent content={msg.content.clone()} is_user={is_user} />
                                            </p>
                                        </div>
                                    </div>
                                }
                            })}

                            if chat_ctx.is_loading {
                                <div class="flex justify-start">
                                    <div class="bg-navy3 border border-border
                                                rounded-xl rounded-tl-sm px-3 py-2">
                                        <div class="flex items-center gap-1">
                                            <div class="w-1 h-1 bg-muted rounded-full animate-bounce"
                                                 style="animation-delay:0ms" />
                                            <div class="w-1 h-1 bg-muted rounded-full animate-bounce"
                                                 style="animation-delay:150ms" />
                                            <div class="w-1 h-1 bg-muted rounded-full animate-bounce"
                                                 style="animation-delay:300ms" />
                                        </div>
                                    </div>
                                </div>
                            }
                        }
                    </div>

                    // Input
                    <div class="border-t border-border px-3 py-3 flex-shrink-0">
                        <div class="flex gap-2">
                            <input
                                type="text"
                                class="input-field text-xs py-2 flex-1"
                                placeholder="Ask a question..."
                                value={(*input).clone()}
                                oninput={on_input}
                                onkeydown={on_key_down}
                                disabled={chat_ctx.is_loading}
                            />
                            <button
                                onclick={on_send}
                                disabled={(*input).trim().is_empty() || chat_ctx.is_loading}
                                class="btn-primary px-3 py-2 flex-shrink-0
                                       disabled:opacity-40 disabled:cursor-not-allowed"
                            >
                                <svg width="12" height="12" viewBox="0 0 24 24" fill="none"
                                     stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                                    <line x1="22" y1="2" x2="11" y2="13"/>
                                    <polygon points="22 2 15 22 11 13 2 9 22 2"/>
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
            }

            // ── Toggle button ─────────────────────────────────────
            <button
                onclick={toggle}
                class={if *open {
                    "w-12 h-12 bg-navy2 border border-orange rounded-full
                     flex items-center justify-center text-orange
                     hover:bg-navy3 transition-all duration-200 shadow-lg"
                } else {
                    "w-12 h-12 bg-orange rounded-full flex items-center justify-center
                     text-white hover:bg-orange2 transition-all duration-200
                     shadow-lg animate-pulse-glow"
                }}
            >
                if *open {
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none"
                         stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                        <line x1="18" y1="6" x2="6" y2="18"/>
                        <line x1="6" y1="6" x2="18" y2="18"/>
                    </svg>
                } else {
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none"
                         stroke="currentColor" stroke-width="2" stroke-linecap="round">
                        <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/>
                    </svg>
                }
            </button>
        </div>
    }
}