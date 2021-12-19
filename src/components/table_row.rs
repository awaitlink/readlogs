use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct TableRowProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub on_click: Callback<MouseEvent>,
}

#[function_component(TableRow)]
pub fn table_row(props: &TableRowProps) -> Html {
    html! {
        <tr class={props.classes.clone()} onclick={props.on_click.clone()}>
            { props.children.clone() }
        </tr>
    }
}
