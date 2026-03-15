use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SearchOverlayProps {
    pub on_close: Callback<()>,
}

#[function_component(SearchOverlay)]
pub fn search_overlay(props: &SearchOverlayProps) -> Html {
    html! { <></> }
}