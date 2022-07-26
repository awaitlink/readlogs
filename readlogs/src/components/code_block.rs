use std::rc::Rc;

use yew::prelude::*;

use crate::components::{Button, ButtonSize};

const LINE_LIMIT_COLLAPSED: usize = 100;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct CodeBlockProps {
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Children,

    #[prop_or_else(|| Rc::new(String::new()))]
    pub text: Rc<String>,
}

#[function_component(CodeBlock)]
pub fn code_block(props: &CodeBlockProps) -> Html {
    let expanded = use_state_eq(|| false);

    let mut classes = classes!(
        props.classes.clone(),
        "rounded-2xl",
        "p-4",
        "overflow-x-auto",
        "text-xs",
    );

    let full_text = Rc::clone(&props.text);

    let (text, footer) = if *expanded {
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
            classes.push(classes!("mb-0", "rounded-b-none"));

            html! {
                <div class={classes!(
                    "max-w-none",
                    "w-full",
                    "rounded-t-none",
                    "rounded-b-2xl",
                    "bg-brand-bg-message",
                    "dark:bg-brand-dark-bg-message",
                    "text-brand-text",
                    "dark:text-brand-dark-text",
                    "text-center",
                    "mx-auto",
                    "p-4",
                )}>
                    { format!("Only the first {} lines are currently shown above. ", LINE_LIMIT_COLLAPSED) }

                    <Button
                        classes={classes!("rounded-2xl")}
                        size={ButtonSize::Small}
                        text="Show all"
                        on_click={Callback::from(move |_| expanded.set(true))}
                    />

                    { " (may take a little while)" }
                </div>
            }
        };

        (Rc::new(text), footer)
    };

    html! {
        <>
            <pre class={classes}>
                <code>
                    { text }
                    { props.children.clone() }
                </code>
            </pre>

            { footer }
        </>
    }
}
