use yew::prelude::*;
use yewtil::NeqAssign;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct IconProps {
    #[prop_or_default]
    pub classes: Classes,

    pub icon: Classes,
}

#[derive(Debug)]
pub struct Icon {
    props: IconProps,
}

impl Component for Icon {
    type Message = ();
    type Properties = IconProps;

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
        let classes = classes!(
            self.props.classes.clone(),
            "w-6",
            "h-6",
            "inline-block",
            "text-center",
        );

        html! {
            <span class=classes>
                <span class=self.props.icon.clone() />
            </span>
        }
    }
}
