use yew::prelude::*;
use crate::types::ChatMessage;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChatState {
    pub messages:   Vec<ChatMessage>,
    pub is_loading: bool,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatAction {
    AddMessage(ChatMessage),
    SetLoading(bool),
    SetSessionId(String),
    Clear,
}

impl Reducible for ChatState {
    type Action = ChatAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut messages   = self.messages.clone();
        let mut is_loading = self.is_loading;
        let mut session_id = self.session_id.clone();

        match action {
            ChatAction::AddMessage(msg) => {
                messages.push(msg);
            }
            ChatAction::SetLoading(val) => {
                is_loading = val;
            }
            ChatAction::SetSessionId(id) => {
                session_id = Some(id);
            }
            ChatAction::Clear => {
                messages.clear();
                session_id = None;
            }
        }

        Self { messages, is_loading, session_id }.into()
    }
}

pub type ChatContext = UseReducerHandle<ChatState>;

#[derive(Properties, PartialEq)]
pub struct ChatProviderProps {
    pub children: Children,
}

#[function_component(ChatProvider)]
pub fn chat_provider(props: &ChatProviderProps) -> Html {
    let chat = use_reducer(ChatState::default);

    html! {
        <ContextProvider<ChatContext> context={chat}>
            { props.children.clone() }
        </ContextProvider<ChatContext>>
    }
}