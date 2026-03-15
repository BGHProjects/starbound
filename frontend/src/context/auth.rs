use yew::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use crate::types::{User, AuthResponse};

const TOKEN_KEY: &str = "starbound_token";
const USER_KEY:  &str = "starbound_user";

#[derive(Debug, Clone, PartialEq)]
pub struct AuthState {
    pub user:       Option<User>,
    pub token:      Option<String>,
    pub is_loading: bool,
}

impl AuthState {
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some() && self.user.is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthAction {
    Login(AuthResponse),
    Logout,
    SetLoading(bool),
}

impl Reducible for AuthState {
    type Action = AuthAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {
            AuthAction::Login(resp) => {
                let _ = LocalStorage::set(TOKEN_KEY, &resp.token);
                let _ = LocalStorage::set(USER_KEY,  &resp.user);
                Self {
                    user:       Some(resp.user),
                    token:      Some(resp.token),
                    is_loading: false,
                }.into()
            }
            AuthAction::Logout => {
                LocalStorage::delete(TOKEN_KEY);
                LocalStorage::delete(USER_KEY);
                Self { user: None, token: None, is_loading: false }.into()
            }
            AuthAction::SetLoading(val) => {
                Self { is_loading: val, ..(*self).clone() }.into()
            }
        }
    }
}

pub type AuthContext = UseReducerHandle<AuthState>;

#[derive(Properties, PartialEq)]
pub struct AuthProviderProps {
    pub children: Children,
}

#[function_component(AuthProvider)]
pub fn auth_provider(props: &AuthProviderProps) -> Html {
    let auth = use_reducer(|| {
        let token: Option<String> = LocalStorage::get(TOKEN_KEY).ok();
        let user:  Option<User>   = LocalStorage::get(USER_KEY).ok();
        AuthState { user, token, is_loading: false }
    });

    html! {
        <ContextProvider<AuthContext> context={auth}>
            { props.children.clone() }
        </ContextProvider<AuthContext>>
    }
}
