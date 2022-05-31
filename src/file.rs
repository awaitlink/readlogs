use std::rc::Rc;

use anyhow::Context;
use yew::prelude::*;

use crate::{
    components::{ButtonSize, CodeBlock, DownloadButton, Message, Title, TitleLevel},
    parsers::*,
    Platform, RemoteObject, RenderedLogSection, SearchQuery, Tab,
};

#[derive(Debug)]
pub struct File {
    remote_object: RemoteObject,
    name: Option<Rc<LogFilename>>,
    text: Rc<String>,
    parsed: anyhow::Result<Content>,
}

impl File {
    pub fn from_text(
        remote_object: RemoteObject,
        name: Option<Rc<LogFilename>>,
        text: String,
    ) -> Self {
        let parsed = Content::parse(remote_object.platform(), &text).context(format!(
            "failed to parse {} debug log file",
            remote_object.platform()
        ));

        Self {
            remote_object,
            name,
            text: Rc::new(text),
            parsed,
        }
    }

    pub fn view(&self, tab: Tab, query: &SearchQuery) -> Html {
        let title = match tab {
            Tab::Information => html! {
                <Title level={TitleLevel::H1} text={format!("{} ({})", tab, self.remote_object.platform())}/>
            },
            Tab::Logs => html! {},
            Tab::Raw => html! {
                <Title level={TitleLevel::H1} text={tab.to_string()}/>
            },
        };

        let content = match tab {
            Tab::Information => match &self.parsed {
                Ok(parsed) => parsed.view_information(self.remote_object.platform()),
                Err(error) => self.view_parsing_error(error),
            },
            Tab::Logs => match &self.parsed {
                Ok(parsed) => RenderedLogSection {
                    title: tab.to_string(),
                    subsections: parsed.view_logs(query),
                    ..Default::default()
                }
                .view(
                    self.remote_object.platform().is_android(),
                    self.remote_object.platform().is_android(),
                    true,
                ),
                Err(error) => self.view_parsing_error(error),
            },
            Tab::Raw => html! {
                <>
                    <DownloadButton
                        classes={classes!("rounded-2xl")}
                        size={ButtonSize::Medium}
                        icon={classes!("fas", "fa-download")}
                        text="Download"
                        content={Rc::clone(&self.text)}
                        filename={format!(
                            "{}-{}{}.txt",
                            self.remote_object.platform(),
                            self.remote_object.key(),
                            self.name.as_ref()
                                .map(|name| format!(
                                    "-{}-{}",
                                    name.app_id,
                                    name.file_time.format("%F-%H-%M-%S-%3f-%Z")
                                ))
                                .unwrap_or_else(|| "".to_owned())
                        ).to_lowercase()}
                    />

                    <CodeBlock text={Rc::clone(&self.text)}/>
                </>
            },
        };

        html! {
            <>
                { title }
                { content }
            </>
        }
    }

    fn view_parsing_error(&self, error: &anyhow::Error) -> Html {
        let notice = "You can still view the raw log by switching to the corresponding tab below"
            .to_owned()
            + match self.remote_object.platform() {
                Platform::Android | Platform::Desktop => ".",
                Platform::Ios => {
                    " or check other files above to see if they were successfully parsed."
                }
            };

        html! {
            <Message error={true} heading="Error parsing file">
                <CodeBlock text={Rc::new(format!("Error: {:?}", error))}/>
                <span class="text-brand-text">{notice}</span>
            </Message>
        }
    }
}
