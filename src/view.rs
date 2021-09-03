use strum::IntoEnumIterator;
use yew::prelude::*;

use crate::{components::*, *};

impl super::Model {
    pub fn view_inner(&self) -> Html {
        let file_picker = match &self.state {
            State::Ready(Object::Multiple(files, active_filename)) => html! {
                <FilePicker
                    classes=classes!("mb-8")
                    files=files.keys().cloned().collect::<Vec<_>>()
                    selected_file=active_filename
                    on_file_selected=self.link.callback(Msg::UpdateActiveFile)
                />
            },
            _ => html! {},
        };

        let active_file = match &self.state {
            State::Ready(_) => self.active_file().view(self.tab, &self.active_query),
            _ => html! {},
        };

        let mut wrapper_classes = classes!("mb-4", "py-4", "bg-white");

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
                <div class=wrapper_classes>
                    <div class="mx-4">
                        { self.view_main_input() }
                        { self.view_help() }

                        { file_picker }
                    </div>

                    <div class="mx-4 prose prose-sm max-w-max mt-8">
                        { active_file }
                    </div>
                </div>

                { self.view_footer() }

                { self.view_display_config() }
            </>
        }
    }

    pub fn view_main_input(&self) -> Html {
        html! {
            <div class="flex mb-8">
                <Input
                    ref=self.debug_log_input.clone()
                    classes=classes!("rounded-l-2xl")
                    value=self.debug_log_url.clone()
                    on_change=self.link.callback(Msg::UpdateUrl)
                    on_submit=self.link.callback(|_| Msg::Start)
                    placeholder="https://debuglogs.org/..."
                    disabled=self.state.is_fetching()
                    autofocus=true
                />

                { self.view_submit_button(ButtonSize::Large) }
            </div>
        }
    }

    pub fn view_help(&self) -> Html {
        match &self.state {
            State::NoData => html! {
                <Message>
                    { "Please enter the debug log URL and press " }
                    <span>{ self.view_submit_button(ButtonSize::Small) }</span>
                    { " or " }
                    <Badge classes=classes!("bg-white") text="Enter ⏎" />
                    { "." }
                </Message>
            },
            State::Fetching(_) => html! {
                <Message
                    heading="Progress".to_owned()
                    text="Fetching and parsing...".to_owned()
                    classes=classes!("animate-pulse")
                />
            },
            State::Error(e) => html! {
                <Message error=true heading="Error".to_owned()>
                    <CodeBlock text=format!("Error: {:?}", e)/>
                </Message>
            },
            _ => html! {},
        }
    }

    pub fn view_submit_button(&self, size: ButtonSize) -> Html {
        html! {
            <Button
                size=size
                on_click=self.link.callback(|_| Msg::Start)
                disabled=self.state.is_fetching()
                text="Read".to_owned()
            />
        }
    }

    pub fn view_display_config(&self) -> Html {
        if !self.state.is_ready() {
            return html! {};
        }

        html! {
            <Toolbar
                classes_outer=classes!("fixed", "bottom-0")
                classes_mid=classes!("rounded-t-2xl")
            >
                <div class="flex flex-col gap-y-2 flex-grow">
                    { self.view_search_toolbar_row() }

                    <div class="flex flex-grow">
                        <div class="flex flex-grow">
                            { for Tab::iter().map(|tab| self.view_tab_button(tab)) }
                        </div>

                        <div>
                            <Button
                                classes=classes!(
                                    "hidden",
                                    "lg:block",
                                    "ml-2",
                                )
                                on_click=self.link.callback(|_| Msg::UpdateUiExpanded)
                                icon=classes!("fas", if self.ui_expanded {
                                    "fa-compress-alt"
                                } else {
                                    "fa-expand-alt"
                                })
                                text=if self.ui_expanded {
                                    "Collapse UI"
                                } else {
                                    "Expand UI"
                                }.to_owned()
                            />
                        </div>
                    </div>
                </div>
            </Toolbar>
        }
    }

    pub fn view_search_toolbar_row(&self) -> Html {
        match (&self.state, &self.tab) {
            (State::Ready(_), Tab::Logs) => {
                let min_log_level_classes = classes!(
                    self.pending_query.min_log_level.color(),
                    "rounded-l-2xl",
                    "border-brand-border",
                    "!border-r-0",
                    "shadow-sm",
                    "focus:border-brand-border",
                    "focus:ring",
                    "focus:ring-brand-focus",
                    "focus:ring-opacity-50",
                    "transition",
                    "duration-200",
                );

                html! {
                    <div class="flex flex-grow">
                        <select
                            value=self.pending_query.min_log_level.to_string()
                            onchange=self.link.callback(|change: ChangeData| match change {
                                ChangeData::Select(data) => Msg::UpdateMinLogLevel(data.value()),
                                _ => unreachable!(),
                            })
                            class=min_log_level_classes
                        >
                            {
                                for LogLevel::iter()
                                    .filter(|variant| variant.applicable_to_platform(self.platform.unwrap()))
                                    .map(|variant| html! {
                                        <option selected={variant == self.pending_query.min_log_level}>{ variant }</option>
                                    })
                            }
                        </select>

                        <Input
                            value=self.pending_query.string.clone()
                            on_change=self.link.callback(Msg::UpdateQuery)
                            on_submit=self.link.callback(|_| Msg::ApplySearchQuery)
                            placeholder={
                                "Search ".to_owned()
                                    + &self.pending_query.min_log_level.to_string().to_lowercase()
                                    + " logs..."
                            }
                        />

                        <Button
                            on_click=self.link.callback(|_| Msg::ApplySearchQuery)
                            icon=classes!("fas", if self.pending_query == self.active_query {
                                "fa-check"
                            } else {
                                "fa-search"
                            })
                            disabled=self.pending_query == self.active_query
                        />
                    </div>
                }
            }
            _ => html! {},
        }
    }

    pub fn view_tab_button(&self, tab: Tab) -> Html {
        html! {
            <Button
                classes=classes!("flex-grow")
                size=ButtonSize::Medium
                on_click=self.link.callback(move |_| Msg::UpdateTab(tab))
                active=self.tab == tab
                icon=tab.icon()
                text=tab.to_string()
            />
        }
    }

    pub fn view_footer(&self) -> Html {
        html! {
            <footer class="bg-brand-bg-footer mb-24 px-8 pb-12 pt-6 text-center">
                <article class="prose prose-sm mx-auto">
                    <p>
                        <Link href="https://github.com/u32i64/readlogs" text="Readlogs" no_referrer=false no_follow=false/>
                        { " is an unofficial project. It is not affilated with the Signal Technology Foundation or Signal Messenger, LLC." }
                    </p>
                    <p><code>{ env!("VERGEN_GIT_SHA_SHORT") }</code></p>
                </article>
            </footer>
        }
    }
}
