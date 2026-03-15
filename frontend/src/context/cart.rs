use yew::prelude::*;
use crate::types::{CartItem, ProductListItem};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CartState {
    pub items: Vec<CartItem>,
}

impl CartState {
    pub fn total(&self) -> f64 {
        self.items.iter().map(|i| i.line_total()).sum()
    }

    pub fn item_count(&self) -> i32 {
        self.items.iter().map(|i| i.quantity).sum()
    }

    pub fn contains(&self, product_id: &str) -> bool {
        self.items.iter().any(|i| i.product.id == product_id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CartAction {
    AddItem(ProductListItem),
    RemoveItem(String),
    UpdateQuantity(String, i32),
    Clear,
}

impl Reducible for CartState {
    type Action = CartAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut items = self.items.clone();

        match action {
            CartAction::AddItem(product) => {
                if let Some(existing) = items.iter_mut().find(|i| i.product.id == product.id) {
                    existing.quantity += 1;
                } else {
                    items.push(CartItem { product, quantity: 1 });
                }
            }
            CartAction::RemoveItem(product_id) => {
                items.retain(|i| i.product.id != product_id);
            }
            CartAction::UpdateQuantity(product_id, qty) => {
                if qty <= 0 {
                    items.retain(|i| i.product.id != product_id);
                } else if let Some(item) = items.iter_mut().find(|i| i.product.id == product_id) {
                    item.quantity = qty;
                }
            }
            CartAction::Clear => items.clear(),
        }

        Self { items }.into()
    }
}

pub type CartContext = UseReducerHandle<CartState>;

#[derive(Properties, PartialEq)]
pub struct CartProviderProps {
    pub children: Children,
}

#[function_component(CartProvider)]
pub fn cart_provider(props: &CartProviderProps) -> Html {
    let cart = use_reducer(CartState::default);

    html! {
        <ContextProvider<CartContext> context={cart}>
            { props.children.clone() }
        </ContextProvider<CartContext>>
    }
}