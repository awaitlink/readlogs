use yew::prelude::*;

use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, InputEvent};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct InputProps {
    #[prop_or_default]
    pub classes: Classes,

    #[prop_or_else(Callback::noop)]
    pub on_change: Callback<String>,
    #[prop_or_else(Callback::noop)]
    pub on_submit_maybe: Callback<bool>, // FIXME: Hacky

    #[prop_or_default]
    pub value: String,
    #[prop_or_default]
    pub placeholder: String,

    #[prop_or(false)]
    pub disabled: bool,
    #[prop_or(false)]
    pub autofocus: bool,
}

#[function_component(Input)]
pub fn input(props: &InputProps) -> Html {
    let classes = classes!(
        props.classes.clone(),
        "z-10",
        "grow",
        "min-w-0",
        "px-4",
        "transition",
        "duration-200",
        "disabled:opacity-50",
        "focus:outline-none",
        "focus:ring",
        "focus:ring-brand-focus",
        "dark:focus:ring-brand-dark-focus",
        "focus:ring-opacity-50",
        "focus:border-brand-focus",
        "dark:focus:border-brand-dark-focus",
        "bg-brand-bg-text-field",
        "dark:bg-brand-dark-bg-text-field",
        "border-brand-border",
        "dark:border-brand-dark-border",
    );

    html! {
        <input
            type={"text"}
            value={props.value.clone()}
            oninput={props.on_change.clone().reform(|event: InputEvent| event.target().unwrap().dyn_into::<HtmlInputElement>().unwrap().value())}
            class={classes}
            placeholder={props.placeholder.clone()}
            onkeypress={props.on_submit_maybe.clone().reform(|e: KeyboardEvent| e.key() == "Enter")}
            disabled={props.disabled}
            autofocus={props.autofocus}
        />
    }
}
