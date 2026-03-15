use yew::prelude::*;
use yew_router::prelude::*;
use crate::context::auth::AuthContext;
use crate::route::Route;

#[derive(Properties, PartialEq)]
pub struct ProtectedRouteProps {
    pub children: Children,
}

#[function_component(ProtectedRoute)]
pub fn protected_route(props: &ProtectedRouteProps) -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    if !auth.is_authenticated() {
        navigator.push(&Route::Login);
        return html! {};
    }

    html! { <>{ props.children.clone() }</> }
}