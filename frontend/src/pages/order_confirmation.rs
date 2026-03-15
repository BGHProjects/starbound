use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct OrderConfirmationProps {
    pub id: String,
}

#[function_component(OrderConfirmation)]
pub fn order_confirmation(props: &OrderConfirmationProps) -> Html {
    html! {
        <div class="min-h-screen bg-navy flex items-center justify-center">
            <h1 class="font-orbitron text-2xl text-orange">{ format!("Order: {}", props.id) }</h1>
        </div>
    }
}