use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SpinnerProps {
    #[prop_or_default]
    pub size: SpinnerSize,
}

#[derive(PartialEq, Default)]
pub enum SpinnerSize {
    Sm,
    #[default]
    Md,
    Lg,
}

#[function_component(Spinner)]
pub fn spinner(props: &SpinnerProps) -> Html {
    let size_class = match props.size {
        SpinnerSize::Sm => "w-4 h-4 border-2",
        SpinnerSize::Md => "w-8 h-8 border-2",
        SpinnerSize::Lg => "w-12 h-12 border-4",
    };

    html! {
        <div class={format!(
            "{} border-navy4 border-t-orange rounded-full animate-spin",
            size_class
        )} />
    }
}