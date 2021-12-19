use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct TableItemProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,

    #[prop_or_else(|| "td".to_owned())]
    pub tag: String,
}

#[function_component(TableItem)]
pub fn table_item(props: &TableItemProps) -> Html {
    let classes = classes!(props.classes.clone(), "first:pl-2", "last:pr-2");

    html! {
        <@{props.tag.clone()} class={classes}>
            { props.children.clone() }
        </@>
    }
}
