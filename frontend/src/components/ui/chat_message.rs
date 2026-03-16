use yew::prelude::*;
use yew_router::prelude::*;
use crate::route::Route;

#[derive(Properties, PartialEq)]
pub struct ChatMessageProps {
    pub content: String,
    pub is_user: bool,
}

// Parses [[Product Name|product_id]] markers into link segments
enum Segment {
    Text(String),
    ProductLink { name: String, id: String },
}

fn parse_segments(content: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut remaining = content;

    while let Some(start) = remaining.find("[[") {
        // Push any text before the marker
        if start > 0 {
            segments.push(Segment::Text(remaining[..start].to_string()));
        }

        let after_open = &remaining[start + 2..];

        if let Some(end) = after_open.find("]]") {
            let inner = &after_open[..end];

            if let Some(pipe) = inner.find('|') {
                let name = inner[..pipe].trim().to_string();
                let id   = inner[pipe + 1..].trim().to_string();
                segments.push(Segment::ProductLink { name, id });
            } else {
                // Malformed marker — treat as plain text
                segments.push(Segment::Text(format!("[[{}]]", inner)));
            }

            remaining = &after_open[end + 2..];
        } else {
            // No closing ]] — treat rest as plain text
            segments.push(Segment::Text(remaining[start..].to_string()));
            remaining = "";
            break;
        }
    }

    if !remaining.is_empty() {
        segments.push(Segment::Text(remaining.to_string()));
    }

    segments
}

#[function_component(ChatMessageContent)]
pub fn chat_message_content(props: &ChatMessageProps) -> Html {
    let segments = parse_segments(&props.content);

    let content_html: Vec<Html> = segments.into_iter().map(|seg| match seg {
        Segment::Text(text) => html! {
            <span style="white-space: pre-wrap;">{ text }</span>
        },
        Segment::ProductLink { name, id } => html! {
            <Link<Route> to={Route::ProductDetail { id: id.clone() }}>
                <span class="text-orange hover:text-orange2 underline underline-offset-2
                             cursor-pointer transition-colors duration-150 font-medium">
                    { name }
                </span>
            </Link<Route>>
        },
    }).collect();

    html! {
        <span>{ for content_html }</span>
    }
}