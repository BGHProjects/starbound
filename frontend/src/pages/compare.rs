use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CompareProps {
    pub id: String,
}

#[function_component(Compare)]
pub fn compare(props: &CompareProps) -> Html {
    html! {
        <div class="min-h-screen bg-navy flex items-center justify-center">
            <h1 class="font-orbitron text-2xl text-orange">{ format!("Compare: {}", props.id) }</h1>
        </div>
    }
}