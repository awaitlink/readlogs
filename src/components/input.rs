use yew::prelude::*;
use yewtil::NeqAssign;

#[derive(Debug, Clone)]
pub enum InputMsg {
    Change(String),
    Submit,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct InputProps {
    #[prop_or_default]
    pub classes: Classes,

    #[prop_or_else(Callback::noop)]
    pub on_change: Callback<String>,
    #[prop_or_else(Callback::noop)]
    pub on_submit: Callback<()>,

    #[prop_or_default]
    pub value: String,
    #[prop_or_default]
    pub placeholder: String,

    #[prop_or(false)]
    pub disabled: bool,
    #[prop_or(false)]
    pub autofocus: bool,
}

#[derive(Debug)]
pub struct Input {
    link: ComponentLink<Self>,
    props: InputProps,
}

impl Component for Input {
    type Message = InputMsg;
    type Properties = InputProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            InputMsg::Change(value) => self.props.on_change.emit(value),
            InputMsg::Submit => self.props.on_submit.emit(()),
        }

        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let classes = classes!(
            self.props.classes.clone(),
            "z-10",
            "flex-grow",
            "min-w-0",
            "px-4",
            "transition",
            "duration-200",
            "disabled:opacity-50",
            "focus:outline-none",
            "focus:ring",
            "focus:ring-brand-focus",
            "focus:ring-opacity-50",
            "focus:border-brand-focus",
        );

        html! {
            <input
                type="text"
                value=self.props.value.clone()
                oninput=self.link.callback(|input: InputData| InputMsg::Change(input.value))
                class=classes
                placeholder=self.props.placeholder.clone()
                onkeypress=self.link.batch_callback(|e: KeyboardEvent| {
                    if e.key() == "Enter" { Some(InputMsg::Submit) } else { None }
                })
                disabled=self.props.disabled
                autofocus=self.props.autofocus
            />
        }
    }
}
