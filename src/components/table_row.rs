use std::convert::identity;

use yew::prelude::*;
use yewtil::NeqAssign;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct TableRowProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub on_click: Callback<MouseEvent>,
}

#[derive(Debug)]
pub struct TableRow {
    link: ComponentLink<Self>,
    props: TableRowProps,
}

impl Component for TableRow {
    type Message = MouseEvent;
    type Properties = TableRowProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.props.on_click.emit(msg);
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <tr class=self.props.classes.clone() onclick=self.link.callback(identity)>
                { self.props.children.clone() }
            </tr>
        }
    }
}
