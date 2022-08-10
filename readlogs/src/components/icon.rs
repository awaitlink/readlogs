use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct IconProps {
    #[prop_or_default]
    pub classes: Classes,

    pub icon: Classes,

    #[prop_or(true)]
    pub fixed_width_height: bool,
}

#[function_component(Icon)]
pub fn icon(props: &IconProps) -> Html {
    let mut classes = classes!(props.classes.clone(), "inline-block", "text-center",);

    if props.fixed_width_height {
        classes.push("w-6");
        classes.push("h-6");
    }

    html! {
        <span class={classes}>
            <span class={props.icon.clone()} />
        </span>
    }
}
