use yew::prelude::*;
use yewtil::NeqAssign;

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

#[derive(Debug)]
pub struct Toolbar {
    link: ComponentLink<Self>,
    props: ToolbarProps,
}

impl Component for Toolbar {
    type Message = ();
    type Properties = ToolbarProps;

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
        let outer = classes!(self.props.classes_outer.clone(), "w-full", "z-20");

        let mid = classes!(
            self.props.classes_mid.clone(),
            "p-2",
            "bg-brand-bg",
            "bg-opacity-50",
            "backdrop-blur",
            "mx-auto",
            "max-w-prose",
        );

        let inner = classes!(
            self.props.classes_inner.clone(),
            "mx-auto",
            "flex",
            "w-[calc(100%-0.25rem)]",
        );

        html! {
            <nav class=outer>
                <div class=mid>
                    <div class=inner>
                        { self.props.children.clone() }
                    </div>
                </div>
            </nav>
        }
    }
}
