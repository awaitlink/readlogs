use derive_more::Display;
use yew::prelude::*;

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

#[function_component(Title)]
pub fn title(props: &TitleProps) -> Html {
    let text = if props.raw {
        html! {
            <code>{ props.text.clone() }</code>
        }
    } else if props.capitalize {
        html! {
            <span class="capitalize">{ props.text.to_lowercase().clone() }</span>
        }
    } else {
        html! { props.text.clone() }
    };

    let meta = match &props.meta {
        Some(meta) => html! {
            <Badge classes={classes!("bg-brand-bg-message", "dark:bg-brand-dark-bg-message", "ml-2")} text={meta.clone()} />
        },
        None => html! {},
    };

    html! {
        <@{props.level.to_string()} class={props.classes.clone()} id={props.id.clone()}>
            { text }{ meta }
        </@>
    }
}
