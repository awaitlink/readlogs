use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct BadgeProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub text: String,
}

#[function_component(Badge)]
pub fn badge(props: &BadgeProps) -> Html {
    let classes = classes!(props.classes.clone(), "rounded-2xl", "px-4", "py-1",);

    html! {
        <span class={classes}>
            { props.text.clone() }
            { props.children.clone() }
        </span>
    }
}
