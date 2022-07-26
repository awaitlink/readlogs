use std::{convert::identity, rc::Rc};

use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlElement};
use yew::prelude::*;

use crate::components::{Button, ButtonSize};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct DownloadButtonProps {
    #[prop_or_default]
    pub classes: Classes,

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

    pub filename: String,
    pub content: Rc<String>,
}

#[derive(Debug)]
pub struct DownloadButton;

impl Component for DownloadButton {
    type Message = MouseEvent;
    type Properties = DownloadButtonProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, _msg: Self::Message) -> bool {
        let document = window().unwrap().document().unwrap();

        let element = document.create_element("a").unwrap();
        element
            .set_attribute(
                "href",
                &format!(
                    "data:text/plain;charset=utf-8,{}",
                    percent_encode(ctx.props().content.as_bytes(), NON_ALPHANUMERIC)
                ),
            )
            .unwrap();
        element
            .set_attribute("download", &ctx.props().filename)
            .unwrap();
        element.set_attribute("style", "display:none;").unwrap();

        document.body().unwrap().append_child(&element).unwrap();
        let element = element.dyn_into::<HtmlElement>().unwrap();
        element.click();
        document.body().unwrap().remove_child(&element).unwrap();

        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <Button
                classes={ctx.props().classes.clone()}
                size={ctx.props().size}
                icon={ctx.props().icon.clone()}
                text={ctx.props().text.clone()}
                active={ctx.props().active}
                disabled={ctx.props().disabled}
                on_click={ctx.link().callback(identity)}
            />
        }
    }
}
