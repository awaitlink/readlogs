use std::convert::identity;

use yew::prelude::*;
use yewtil::NeqAssign;

use crate::components::{Icon, Link};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonSize {
    Large,
    Medium,
    Small,
}

impl Default for ButtonSize {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct ButtonProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_else(Callback::noop)]
    pub on_click: Callback<MouseEvent>,

    #[prop_or_default]
    pub size: ButtonSize,

    #[prop_or_default]
    pub icon: Option<Classes>,
    #[prop_or_default]
    pub text: Option<String>,

    #[prop_or(false)]
    pub active: bool,
    #[prop_or(false)]
    pub disabled: bool,

    #[prop_or_default]
    pub href: Option<String>,
}

#[derive(Debug)]
pub struct Button {
    link: ComponentLink<Self>,
    props: ButtonProps,
}

impl Component for Button {
    type Message = MouseEvent;
    type Properties = ButtonProps;

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
        let mut classes = classes!(
            self.props.classes.clone(),
            "transition",
            "duration-200",
            "px-4",
            "bg-brand-bg",
            "dark:bg-brand-dark-bg",
            "text-brand-text",
            "dark:text-brand-dark-text",
            "disabled:cursor-not-allowed",
            "disabled:hover:bg-opacity-0",
            "disabled:hover:text-brand-text",
            "dark:disabled:hover:text-brand-dark-text",
            "disabled:opacity-50",
            "border",
            "border-brand-border",
            "dark:border-brand-dark-border",
            "hover:bg-brand-primary-hover",
            "dark:hover:bg-brand-dark-primary-hover",
            "hover:text-brand-text-primary-hover",
            "dark:hover:text-brand-dark-text-primary-hover",
            "focus:outline-none",
            "focus:ring",
            "focus:ring-brand-focus",
            "dark:focus:ring-brand-dark-focus",
            "focus:ring-opacity-50",
            "focus:border-brand-focus",
            "dark:focus:border-brand-dark-focus",
            "first:rounded-l-2xl",
            "first:border-r-0",
            "last:rounded-r-2xl",
            "last:border-l-0",
            "first:last:border",
        );

        classes.push(match self.props.size {
            ButtonSize::Large => {
                classes!("text-lg", "py-2")
            }
            ButtonSize::Medium => {
                classes!("py-2")
            }
            ButtonSize::Small => {
                classes!("text-sm", "py-1")
            }
        });

        classes.push(if self.props.active {
            classes!(
                "bg-brand-primary-active",
                "dark:bg-brand-dark-primary-active",
                "text-brand-text-primary-active",
                "dark:text-brand-dark-text-primary-active"
            )
        } else {
            classes!()
        });

        let inner = match (&self.props.icon, &self.props.text) {
            (Some(icon), Some(text)) => html! {
                <>
                    <Icon icon=icon.clone() classes=classes!("mr-2") />
                    { text.clone() }
                </>
            },
            (Some(icon), None) => html! { <Icon icon=icon.clone() /> },
            (None, Some(text)) => html! { text.clone() },
            (None, None) => html! {},
        };

        let button = html! {
            <button
                class=classes
                onclick=self.link.callback(identity)
                disabled=self.props.disabled
            >
                { inner }
            </button>
        };

        match &self.props.href {
            Some(href) => html! {
                <Link href=href.clone()>
                    { button }
                </Link>
            },
            None => button,
        }
    }
}
