use yew::prelude::*;

#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
        <div class="min-h-screen bg-navy flex items-center justify-center">
            <div class="text-center">
                <h1 class="font-orbitron text-6xl font-bold text-orange mb-4">{"404"}</h1>
                <p class="text-muted font-exo text-lg">{"Page not found"}</p>
            </div>
        </div>
    }
}