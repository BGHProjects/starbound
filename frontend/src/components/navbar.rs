use yew::prelude::*;

#[function_component(Navbar)]
pub fn navbar() -> Html {
    html! {
        <nav class="bg-navy2 border-b border-border px-6 h-16 flex items-center justify-between">
            <span class="font-orbitron text-lg font-bold text-white">
                {"STAR"}
                <span class="text-orange">{"BOUND"}</span>
            </span>
            <div class="flex items-center gap-4">
                <a href="/catalog" class="text-muted hover:text-white text-sm font-medium transition-colors">{"Catalog"}</a>
                <a href="/account" class="text-muted hover:text-white text-sm font-medium transition-colors">{"Account"}</a>
                <button class="btn-primary text-sm">{"Cart"}</button>
            </div>
        </nav>
    }
}