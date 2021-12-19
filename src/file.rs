use std::rc::Rc;

use anyhow::Context;
use yew::prelude::*;

use crate::{
    components::{ButtonSize, CodeBlock, DownloadButton, Title, TitleLevel},
    parsers::*,
    RemoteObject, RenderedLogSection, SearchQuery, Tab,
};

#[derive(Debug, PartialEq)]
pub struct File {
    remote_object: RemoteObject,
    name: Option<Rc<LogFilename>>,
    text: String,
    parsed: Content,
}

impl File {
    pub fn from_text(
        remote_object: RemoteObject,
        name: Option<Rc<LogFilename>>,
        text: String,
    ) -> anyhow::Result<Self> {
        let parsed = Content::parse(remote_object.platform(), &text).context(format!(
            "failed to parse {} debug log file",
            remote_object.platform()
        ))?;

        Ok(Self {
            remote_object,
            name,
            text,
            parsed,
        })
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
            Tab::Information => self.parsed.view_information(self.remote_object.platform()),
            Tab::Logs => RenderedLogSection {
                title: tab.to_string(),
                subsections: self.parsed.view_logs(query),
                ..Default::default()
            }
            .view(
                self.remote_object.platform().is_android(),
                self.remote_object.platform().is_android(),
                true,
            ),
            Tab::Raw => html! {
                <>
                    <DownloadButton
                        classes={classes!("rounded-2xl")}
                        size={ButtonSize::Medium}
                        icon={classes!("fas", "fa-download")}
                        text={"Download".to_owned()}
                        content={self.text.clone()}
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

                    <CodeBlock text={self.text.clone()}/>
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
}
