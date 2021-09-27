use yew::prelude::*;
use yewtil::NeqAssign;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct TableItemProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,

    #[prop_or_else(|| "td".to_owned())]
    pub tag: String,
}

#[derive(Debug)]
pub struct TableItem {
    props: TableItemProps,
}

impl Component for TableItem {
    type Message = ();
    type Properties = TableItemProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let classes = classes!(self.props.classes.clone(), "first:pl-2", "last:pr-2");

        html! {
            <@{self.props.tag.clone()} class=classes>
                { self.props.children.clone() }
            </@>
        }
    }
}
