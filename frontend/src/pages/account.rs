use yew::prelude::*;

#[function_component(Account)]
pub fn account() -> Html {
    html! {
        <div class="p-8">
            <h1 class="font-orbitron text-2xl text-orange">{"Account"}</h1>
            <p class="text-muted mt-2">{"Account page"}</p>
        </div>
    }
}