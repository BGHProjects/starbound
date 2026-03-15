use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct RefundProps {
    pub order_id: String,
}

#[function_component(Refund)]
pub fn refund(props: &RefundProps) -> Html {
    html! {
        <div class="min-h-screen bg-navy flex items-center justify-center">
            <h1 class="font-orbitron text-2xl text-orange">{ format!("Refund: {}", props.order_id) }</h1>
        </div>
    }
}