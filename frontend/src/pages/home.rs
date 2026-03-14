use yew::prelude::*;

#[function_component(Home)]
pub fn home() -> Html {
    html! {
        <div class="p-8">
            <h1 class="font-orbitron text-2xl text-orange">{"Starbound"}</h1>
            <p class="text-muted mt-2">{"Home page"}</p>
        </div>
    }
}