use yew::prelude::*;
use yewtil::NeqAssign;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct TableProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,
}

#[derive(Debug)]
pub struct Table {
    link: ComponentLink<Self>,
    props: TableProps,
}

impl Component for Table {
    type Message = ();
    type Properties = TableProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let classes = classes!(
            self.props.classes.clone(),
            "max-w-max",
            "!my-0",
            "!leading-tight",
        );

        html! {
            <div class="overflow-x-auto">
                <table class=classes>
                    { self.props.children.clone() }
                </table>
            </div>
        }
    }
}
