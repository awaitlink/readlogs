use std::rc::Rc;

use strum::IntoEnumIterator;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlSelectElement};
use yew::prelude::*;

use crate::{components::*, *};

impl super::Model {
    pub fn view_inner(&self, ctx: &Context<Self>) -> Html {
        let file_picker = match &self.state {
            State::Ready(Object::Multiple {
                files,
                active_filename,
            }) => html! {
                <FilePicker
                    classes={classes!("mb-8")}
                    files={files.keys().cloned().collect::<Vec<_>>()}
                    selected_file={active_filename}
                    on_file_selected={ctx.link().callback(Msg::UpdateActiveFile)}
                />
            },
            _ => html! {},
        };

        let active_file = match &self.state {
            State::Ready(_) => self.active_file().view(self.tab, &self.active_query),
            _ => html! {},
        };

        let mut wrapper_classes = classes!("mb-4", "py-4", "bg-brand-bg", "dark:bg-brand-dark-bg");

        if !self.ui_expanded {
            wrapper_classes.push(classes!(
                "max-w-5xl",
                "mx-auto",
                "lg:mt-4",
                "lg:rounded-2xl",
            ));
        }

        html! {
            <>
                <div class={wrapper_classes}>
                    <div class="mx-4">
                        { self.view_main_input(ctx) }
                        { self.view_help(ctx) }

                        { file_picker }
                    </div>

                    <div class="mx-4 prose dark:prose-invert prose-sm max-w-max mt-8">
                        { active_file }
                    </div>
                </div>

                { self.view_footer() }

                { self.view_display_config(ctx) }
            </>
        }
    }

    pub fn view_main_input(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="flex mb-8">
                <Input
                    ref={self.debug_log_input.clone()}
                    classes={classes!("rounded-l-2xl")}
                    value={self.debug_log_url.clone()}
                    on_change={ctx.link().callback(Msg::UpdateUrl)}
                    on_submit_maybe={ctx.link().batch_callback(|actually: bool| actually.then_some(Msg::Start))}
                    placeholder="https://debuglogs.org/..."
                    disabled={self.state.is_fetching()}
                    autofocus={true}
                />

                { self.view_submit_button(ButtonSize::Large, ctx) }
            </div>
        }
    }

    pub fn view_help(&self, ctx: &Context<Self>) -> Html {
        match &self.state {
            State::NoData => html! {
                <Message>
                    { "Please enter a Signal " }
                    <Link href="https://support.signal.org/hc/en-us/articles/360007318591" text="debug log"/>
                    {" URL and press " }
                    <span>{ self.view_submit_button(ButtonSize::Small, ctx) }</span>
                    { " or " }
                    <Badge classes={classes!("bg-brand-bg", "dark:bg-brand-dark-bg")} text="Enter ⏎" />
                    { "." }
                </Message>
            },
            State::Fetching => html! {
                <Message
                    heading="Progress"
                    text="Fetching and parsing..."
                    classes={classes!("animate-pulse")}
                />
            },
            State::Error(e) => html! {
                <Message error={true} heading="Error">
                    <CodeBlock text={Rc::new(format!("Error: {:?}", e))}/>
                </Message>
            },
            _ => html! {},
        }
    }

    pub fn view_submit_button(&self, size: ButtonSize, ctx: &Context<Self>) -> Html {
        html! {
            <Button
                {size}
                on_click={ctx.link().callback(|_| Msg::Start)}
                disabled={self.state.is_fetching()}
                text="Read"
            />
        }
    }

    pub fn view_display_config(&self, ctx: &Context<Self>) -> Html {
        if !self.state.is_ready() {
            return html! {};
        }

        html! {
            <Toolbar
                classes_outer={classes!("fixed", "bottom-0")}
                classes_mid={classes!("rounded-t-2xl")}
            >
                <div class="flex flex-col gap-y-2 grow">
                    { self.view_search_toolbar_row(ctx) }

                    <div class="flex grow">
                        <div class="flex grow">
                            { for Tab::iter().map(|tab| self.view_tab_button(tab, ctx)) }
                        </div>

                        <div>
                            <Button
                                classes={classes!(
                                    "hidden",
                                    "lg:block",
                                    "ml-2",
                                )}
                                on_click={ctx.link().callback(|_| Msg::UpdateUiExpanded)}
                                icon={classes!("fas", if self.ui_expanded {
                                    "fa-compress-alt"
                                } else {
                                    "fa-expand-alt"
                                })}
                                text={if self.ui_expanded {
                                    "Collapse UI"
                                } else {
                                    "Expand UI"
                                }.to_owned()}
                            />
                        </div>
                    </div>
                </div>
            </Toolbar>
        }
    }

    pub fn view_search_toolbar_row(&self, ctx: &Context<Self>) -> Html {
        match (&self.state, &self.tab) {
            (State::Ready(_), Tab::Logs) => {
                let min_log_level_classes = classes!(
                    self.pending_query.min_log_level.color(),
                    "rounded-l-2xl",
                    "border-brand-border",
                    "dark:border-brand-dark-border",
                    "!border-r-0",
                    "shadow-sm",
                    "focus:border-brand-border",
                    "dark:focus:border-brand-dark-border",
                    "focus:ring",
                    "focus:ring-brand-focus",
                    "dark:focus:ring-brand-dark-focus",
                    "focus:ring-opacity-50",
                    "transition",
                    "duration-200",
                    "bg-brand-bg",
                    "dark:bg-brand-dark-bg",
                );

                html! {
                    <div class="flex grow">
                        <select
                            value={self.pending_query.min_log_level.to_string()}
                            onchange={ctx.link().callback(|event: Event|
                                Msg::UpdateMinLogLevel(event.target().unwrap().dyn_into::<HtmlSelectElement>().unwrap().value())
                            )}
                            class={min_log_level_classes}
                        >
                            {
                                for LogLevel::iter()
                                    .filter(|variant| variant.applicable_to_platform(self.remote_object.as_ref().unwrap().platform()))
                                    .map(|variant| html! {
                                        <option selected={variant == self.pending_query.min_log_level}>{ variant }</option>
                                    })
                            }
                        </select>

                        <Input
                            value={self.pending_query.string.clone()}
                            on_change={ctx.link().callback(Msg::UpdateQuery)}
                            on_submit_maybe={ctx.link().batch_callback(|actually: bool| actually.then_some(Msg::ApplySearchQuery))}
                            placeholder={
                                "Search ".to_owned()
                                    + &self.pending_query.min_log_level.to_string().to_lowercase()
                                    + " logs..."
                            }
                        />

                        <Button
                            on_click={ctx.link().callback(|_| Msg::ApplySearchQuery)}
                            icon={classes!("fas", if self.pending_query == self.active_query {
                                "fa-check"
                            } else {
                                "fa-search"
                            })}
                            disabled={self.pending_query == self.active_query}
                        />
                    </div>
                }
            }
            _ => html! {},
        }
    }

    pub fn view_tab_button(&self, tab: Tab, ctx: &Context<Self>) -> Html {
        html! {
            <Button
                classes={classes!("grow")}
                size={ButtonSize::Medium}
                on_click={ctx.link().callback(move |_| Msg::UpdateTab(tab))}
                active={self.tab == tab}
                icon={tab.icon()}
                text={tab.to_string()}
            />
        }
    }

    pub fn view_footer(&self) -> Html {
        html! {
            <footer class="bg-brand-bg-footer dark:bg-brand-dark-bg-footer mb-24 px-8 pb-12 pt-6 text-center">
                <article class="prose dark:prose-invert prose-sm mx-auto">
                    <p>
                        <Link href="https://github.com/u32i64/readlogs" text="Readlogs" no_referrer={false} no_follow={false}/>
                        { " is an unofficial project. It is not affiliated with the Signal Technology Foundation or Signal Messenger, LLC." }
                    </p>
                    <p><code>{ env!("VERGEN_GIT_SHA_SHORT") }</code></p>
                </article>
            </footer>
        }
    }
}
