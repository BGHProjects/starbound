use yew::prelude::*;

#[function_component(Checkout)]
pub fn checkout() -> Html {
    html! {
        <div class="p-8">
            <h1 class="font-orbitron text-2xl text-orange">{"Checkout"}</h1>
            <p class="text-muted mt-2">{"Checkout page"}</p>
        </div>
    }
}