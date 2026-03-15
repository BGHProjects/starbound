use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TooltipProps {
    pub text:       String,
    #[prop_or_default]
    pub link:       Option<String>,
    #[prop_or_default]
    pub link_label: Option<String>,
}

#[function_component(Tooltip)]
pub fn tooltip(props: &TooltipProps) -> Html {
    let visible    = use_state(|| false);
    let text       = props.text.clone();
    let link       = props.link.clone();
    let link_label = props.link_label.clone();

    let on_enter = {
        let visible = visible.clone();
        Callback::from(move |_: MouseEvent| visible.set(true))
    };

    let on_leave = {
        let visible = visible.clone();
        Callback::from(move |_: MouseEvent| visible.set(false))
    };

    let bubble = if *visible {
        html! {
            <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 z-50
                        w-60 bg-navy2 border border-border rounded-xl p-3" style="font-family: 'Exo 2', sans-serif;">
                <div class="absolute top-full left-1/2 -translate-x-1/2
                            border-4 border-transparent border-t-border"
                     style="margin-top:-1px;">
                </div>
                <p class="font-exo text-xs text-muted leading-relaxed">{ &text }</p>
                {
                    if let Some(href) = &link {
                        html! {
                            <a href={href.clone()}
                               target="_blank"
                               rel="noopener noreferrer"
                               class="inline-block mt-2 font-exo text-xs text-orange
                                      hover:text-orange2 transition-colors">
                                { link_label.clone().unwrap_or_else(|| "Learn more →".to_string()) }
                            </a>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    } else {
        html! {}
    };

    html! {
        // Wrapper covers both icon and bubble — mouseleave only fires
        // when the cursor leaves the whole group, not when moving between them
        <span
            class="relative inline-flex items-center ml-1.5"
            style="vertical-align:middle;"
            onmouseenter={on_enter}
            onmouseleave={on_leave}
        >
            <span
                class="w-4 h-4 rounded-full border border-dim text-dim flex items-center
                       justify-center cursor-help hover:border-orange hover:text-orange
                       transition-colors duration-150 font-exo text-xs font-bold
                       leading-none select-none flex-shrink-0"
            >
                {"i"}
            </span>
            { bubble }
        </span>
    }
}