use yew::prelude::*;
use yewtil::NeqAssign;

use crate::components::{Title, TitleLevel};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct MessageProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub heading: Option<String>,
    #[prop_or_default]
    pub text: Option<String>,

    #[prop_or(false)]
    pub error: bool,
}

#[derive(Debug)]
pub struct Message {
    props: MessageProps,
}

impl Component for Message {
    type Message = ();
    type Properties = MessageProps;

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
        let mut classes = classes!(
            self.props.classes.clone(),
            "p-4",
            "rounded-2xl",
            "border-brand-border",
            "dark:border-brand-dark-border",
            "prose",
            "dark:prose-invert",
            "prose-sm",
            "max-w-max",
            "mx-auto",
        );

        classes.push(if self.props.error {
            classes!("bg-red-100")
        } else {
            classes!("bg-brand-bg-message", "dark:bg-brand-dark-bg-message")
        });

        let heading = match &self.props.heading {
            Some(heading) => {
                let heading_classes = if self.props.error {
                    classes!("!text-red-600")
                } else {
                    classes!()
                };

                html! {
                    <Title level=TitleLevel::H2 classes=heading_classes text=heading.clone() />
                }
            }
            None => html! {},
        };

        let text = match &self.props.text {
            Some(text) => html! {
                <p>{ text.clone() }</p>
            },
            None => html! {},
        };

        html! {
            <div class=classes>
                { heading }
                { text }
                { self.props.children.clone() }
            </div>
        }
    }
}
