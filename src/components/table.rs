use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct TableProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Table)]
pub fn table(props: &TableProps) -> Html {
    let classes = classes!(
        props.classes.clone(),
        "max-w-max",
        "!my-0",
        "!leading-tight",
    );

    html! {
        <div class="overflow-x-auto">
            <table class={classes}>
                { props.children.clone() }
            </table>
        </div>
    }
}
