use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct LinkProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,

    #[prop_or("#".to_owned())]
    pub href: String,
    #[prop_or_default]
    pub text: String,

    #[prop_or(true)]
    pub no_referrer: bool,
    #[prop_or(true)]
    pub no_follow: bool,

    #[prop_or(true)]
    pub new_tab: bool,
}

#[function_component(Link)]
pub fn link(props: &LinkProps) -> Html {
    let mut rel = classes!("noopener");

    if props.no_referrer {
        rel.push("noreferrer");
    }

    if props.no_follow {
        rel.push("nofollow");
    }

    html! {
        <a
            class={props.classes.clone()}
            href={props.href.clone()}
            target={props.new_tab.then(|| "_blank")}
            {rel}
        >
            { &props.text }
            { props.children.clone() }
        </a>
    }
}
