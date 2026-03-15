use yew::prelude::*;

#[function_component(Landing)]
pub fn landing() -> Html {
    html! {
        <div class="min-h-screen bg-navy flex items-center justify-center">
            <h1 class="font-orbitron text-2xl text-orange">{"Landing"}</h1>
        </div>
    }
}