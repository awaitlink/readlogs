use yew::prelude::*;
use yewtil::NeqAssign;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct CodeBlockProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub text: String,
}

#[derive(Debug)]
pub struct CodeBlock {
    props: CodeBlockProps,
}

impl Component for CodeBlock {
    type Message = ();
    type Properties = CodeBlockProps;

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
        let classes = classes!(
            self.props.classes.clone(),
            "rounded-2xl",
            "p-4",
            "overflow-x-auto",
            "text-xs",
        );

        html! {
            <pre class=classes>
                <code>
                    { self.props.text.clone() }
                    { self.props.children.clone() }
                </code>
            </pre>
        }
    }
}
