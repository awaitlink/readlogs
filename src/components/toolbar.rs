use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct ToolbarProps {
    #[prop_or_default]
    pub classes_outer: Classes,
    #[prop_or_default]
    pub classes_mid: Classes,
    #[prop_or_default]
    pub classes_inner: Classes,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Toolbar)]
pub fn toolbar(props: &ToolbarProps) -> Html {
    let outer = classes!(props.classes_outer.clone(), "w-full", "z-20");

    let mid = classes!(
        props.classes_mid.clone(),
        "p-2",
        "bg-brand-bg",
        "dark:bg-brand-dark-bg",
        "bg-opacity-50",
        "dark:bg-opacity-50",
        "backdrop-blur",
        "mx-auto",
        "max-w-prose",
    );

    let inner = classes!(
        props.classes_inner.clone(),
        "mx-auto",
        "flex",
        "w-[calc(100%-0.25rem)]",
    );

    html! {
        <nav class={outer}>
            <div class={mid}>
                <div class={inner}>
                    { props.children.clone() }
                </div>
            </div>
        </nav>
    }
}
