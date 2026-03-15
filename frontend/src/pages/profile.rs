use yew::prelude::*;

#[function_component(Profile)]
pub fn profile() -> Html {
    html! {
        <div class="min-h-screen bg-navy flex items-center justify-center">
            <h1 class="font-orbitron text-2xl text-orange">{"Profile"}</h1>
        </div>
    }
}