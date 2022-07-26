use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct IconProps {
    #[prop_or_default]
    pub classes: Classes,

    pub icon: Classes,
}

#[function_component(Icon)]
pub fn icon(props: &IconProps) -> Html {
    let classes = classes!(
        props.classes.clone(),
        "w-6",
        "h-6",
        "inline-block",
        "text-center",
    );

    html! {
        <span class={classes}>
            <span class={props.icon.clone()} />
        </span>
    }
}
