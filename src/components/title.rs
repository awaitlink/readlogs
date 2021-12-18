use derive_more::Display;
use yew::prelude::*;
use yewtil::NeqAssign;

use crate::components::Badge;

#[derive(Debug, Display, Clone, Copy, PartialEq, PartialOrd)]
pub enum TitleLevel {
    H1,
    H2,
    H3,
    H4,
}

impl TitleLevel {
    pub fn incremented(&self) -> Option<Self> {
        match self {
            TitleLevel::H1 => Some(TitleLevel::H2),
            TitleLevel::H2 => Some(TitleLevel::H3),
            TitleLevel::H3 => Some(TitleLevel::H4),
            TitleLevel::H4 => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct TitleProps {
    #[prop_or_default]
    pub classes: Classes,

    #[prop_or(TitleLevel::H1)]
    pub level: TitleLevel,

    #[prop_or_default]
    pub text: String,
    #[prop_or_default]
    pub meta: Option<String>,

    #[prop_or_default]
    pub id: Option<String>,

    #[prop_or(false)]
    pub raw: bool,
    #[prop_or(false)]
    pub capitalize: bool,
}

#[derive(Debug)]
pub struct Title {
    props: TitleProps,
}

impl Component for Title {
    type Message = ();
    type Properties = TitleProps;

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
        let text = if self.props.raw {
            html! {
                <code>{ self.props.text.clone() }</code>
            }
        } else if self.props.capitalize {
            html! {
                <span class="capitalize">{ self.props.text.to_lowercase().clone() }</span>
            }
        } else {
            html! { self.props.text.clone() }
        };

        let meta = match &self.props.meta {
            Some(meta) => html! {
                <Badge classes=classes!("bg-brand-bg-message", "dark:bg-brand-dark-bg-message", "ml-2") text=meta.clone() />
            },
            None => html! {},
        };

        html! {
            <@{self.props.level.to_string()} class=self.props.classes.clone() id=self.props.id.clone()>
                { text }{ meta }
            </@>
        }
    }
}
