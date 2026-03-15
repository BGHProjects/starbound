use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProductDetailProps {
    pub id: String,
}

#[function_component(ProductDetail)]
pub fn product_detail(props: &ProductDetailProps) -> Html {
    html! {
        <div class="min-h-screen bg-navy flex items-center justify-center">
            <h1 class="font-orbitron text-2xl text-orange">{ format!("Product: {}", props.id) }</h1>
        </div>
    }
}
