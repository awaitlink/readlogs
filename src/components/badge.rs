use yew::prelude::*;
use yewtil::NeqAssign;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct BadgeProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub text: String,
}

#[derive(Debug)]
pub struct Badge {
    props: BadgeProps,
}

impl Component for Badge {
    type Message = ();
    type Properties = BadgeProps;

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
        let classes = classes!(self.props.classes.clone(), "rounded-2xl", "px-4", "py-1",);

        html! {
            <span class=classes>
                { self.props.text.clone() }
                { self.props.children.clone() }
            </span>
        }
    }
}
