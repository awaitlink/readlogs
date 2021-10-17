use std::convert::identity;

use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use wasm_bindgen::JsCast;
use yew::{prelude::*, utils::document, web_sys::HtmlElement};
use yewtil::NeqAssign;

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
    pub content: String,
}

#[derive(Debug)]
pub struct DownloadButton {
    link: ComponentLink<Self>,
    props: DownloadButtonProps,
}

impl Component for DownloadButton {
    type Message = MouseEvent;
    type Properties = DownloadButtonProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        let element = document().create_element("a").unwrap();
        element
            .set_attribute(
                "href",
                &format!(
                    "data:text/plain;charset=utf-8,{}",
                    percent_encode(self.props.content.as_bytes(), NON_ALPHANUMERIC)
                ),
            )
            .unwrap();
        element
            .set_attribute("download", &self.props.filename)
            .unwrap();
        element.set_attribute("style", "display:none;").unwrap();

        document().body().unwrap().append_child(&element).unwrap();
        let element = element.dyn_into::<HtmlElement>().unwrap();
        element.click();
        document().body().unwrap().remove_child(&element).unwrap();

        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <Button
                classes=self.props.classes.clone()
                size=self.props.size
                icon=self.props.icon.clone()
                text=self.props.text.clone()
                active=self.props.active
                disabled=self.props.disabled
                on_click=self.link.callback(identity)
            />
        }
    }
}
