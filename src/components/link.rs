use yew::prelude::*;
use yewtil::NeqAssign;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct LinkProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,

    #[prop_or("#".to_owned())]
    pub href: String,
    #[prop_or_default]
    pub text: String,

    #[prop_or(true)]
    pub no_referrer: bool,
    #[prop_or(true)]
    pub no_follow: bool,

    #[prop_or(true)]
    pub new_tab: bool,
}

#[derive(Debug)]
pub struct Link {
    link: ComponentLink<Self>,
    props: LinkProps,
}

impl Component for Link {
    type Message = ();
    type Properties = LinkProps;

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
        let mut rel = classes!("noopener");

        if self.props.no_referrer {
            rel.push("noreferrer");
        }

        if self.props.no_follow {
            rel.push("nofollow");
        }

        html! {
            <a
                class=self.props.classes.clone()
                href=self.props.href.clone()
                target=self.props.new_tab.then(|| "_blank")
                rel=rel
            >
                { self.props.text.clone() }
                { self.props.children.clone() }
            </a>
        }
    }
}
