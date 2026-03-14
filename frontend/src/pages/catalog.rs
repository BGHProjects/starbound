use yew::prelude::*;

#[function_component(Catalog)]
pub fn catalog() -> Html {
    html! {
        <div class="p-8">
            <h1 class="font-orbitron text-2xl text-orange">{"Catalog"}</h1>
            <p class="text-muted mt-2">{"Product catalog"}</p>
        </div>
    }
}