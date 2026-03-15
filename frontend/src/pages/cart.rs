use yew::prelude::*;

#[function_component(Cart)]
pub fn cart() -> Html {
    html! {
        <div class="min-h-screen bg-navy flex items-center justify-center">
            <h1 class="font-orbitron text-2xl text-orange">{"Cart"}</h1>
        </div>
    }
}