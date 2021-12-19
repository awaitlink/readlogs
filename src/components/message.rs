use yew::prelude::*;

use crate::components::{Title, TitleLevel};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct MessageProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub heading: Option<String>,
    #[prop_or_default]
    pub text: Option<String>,

    #[prop_or(false)]
    pub error: bool,
}

#[function_component(Message)]
pub fn message(props: &MessageProps) -> Html {
    let mut classes = classes!(
        props.classes.clone(),
        "p-4",
        "rounded-2xl",
        "border-brand-border",
        "dark:border-brand-dark-border",
        "prose",
        "dark:prose-invert",
        "prose-sm",
        "max-w-max",
        "mx-auto",
    );

    classes.push(if props.error {
        classes!("bg-red-100")
    } else {
        classes!("bg-brand-bg-message", "dark:bg-brand-dark-bg-message")
    });

    let heading = match &props.heading {
        Some(heading) => {
            let heading_classes = if props.error {
                classes!("!text-red-600")
            } else {
                classes!()
            };

            html! {
                <Title level={TitleLevel::H2} classes={heading_classes} text={heading.clone()} />
            }
        }
        None => html! {},
    };

    let text = match &props.text {
        Some(text) => html! {
            <p>{ text }</p>
        },
        None => html! {},
    };

    html! {
        <div class={classes}>
            { heading }
            { text }
            { props.children.clone() }
        </div>
    }
}
