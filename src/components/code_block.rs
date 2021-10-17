use yew::prelude::*;
use yewtil::NeqAssign;

use crate::components::{Button, ButtonSize};

const LINE_LIMIT_COLLAPSED: usize = 100;

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
    link: ComponentLink<Self>,
    expanded: bool,
}

impl Component for CodeBlock {
    type Message = ();
    type Properties = CodeBlockProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            expanded: false,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        self.expanded = !self.expanded;
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.expanded = false;
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

        let full_text = self.props.text.clone();

        let (text, footer) = if self.expanded {
            (full_text, html! {})
        } else {
            let text = full_text
                .split('\n')
                .take(LINE_LIMIT_COLLAPSED)
                .collect::<Vec<_>>()
                .join("\n");

            let footer = if text.len() == full_text.len() {
                html! {}
            } else {
                html! {
                    <div class=classes!(
                        "m-[-1.6666667em]",
                        "max-w-none",
                        "w-full",
                        "rounded-t-none",
                        "rounded-b-2xl",
                        "bg-brand-text",
                        "text-brand-text-primary-active",
                        "text-center",
                        "mx-auto",
                        "p-4",
                    )>
                        { format!("Only the first {} lines are currently shown above. ", LINE_LIMIT_COLLAPSED) }

                        <Button
                            classes=classes!("rounded-2xl")
                            size=ButtonSize::Small
                            text="Show all".to_owned()
                            on_click=self.link.callback(|_| ())
                        />

                        { " (may take a little while)" }
                    </div>
                }
            };

            (text, footer)
        };

        html! {
            <>
                <pre class=classes>
                    <code>
                        { text }
                        { self.props.children.clone() }
                    </code>
                </pre>

                { footer }
            </>
        }
    }
}
