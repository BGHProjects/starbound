use yew::prelude::*;

#[function_component(Catalog)]
pub fn catalog() -> Html {
    html! {
        <div class="min-h-screen bg-navy flex items-center justify-center">
            <h1 class="font-orbitron text-2xl text-orange">{"Catalog"}</h1>
        </div>
    }
}