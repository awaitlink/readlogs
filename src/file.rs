use anyhow::Context;
use yew::prelude::*;

use crate::{
    components::{CodeBlock, Title, TitleLevel},
    parsers::*,
    Platform, RenderedLogSection, SearchQuery, Tab,
};

#[derive(Debug, PartialEq)]
pub struct File {
    platform: Platform,
    text: String,
    parsed: Content,
}

impl File {
    pub fn from_text(platform: Platform, text: String) -> anyhow::Result<Self> {
        let parsed = Content::parse(platform, &text)
            .context(format!("failed to parse {} debug log file", platform))?;

        Ok(Self {
            platform,
            text,
            parsed,
        })
    }

    pub fn view(&self, tab: Tab, query: &SearchQuery) -> Html {
        let title = match tab {
            Tab::Information => html! {
                <Title level=TitleLevel::H1 text=format!("{} ({})", tab, self.platform)/>
            },
            Tab::Logs => html! {},
            Tab::Raw => html! {
                <Title level=TitleLevel::H1 text=tab.to_string()/>
            },
        };

        let content = match tab {
            Tab::Information => self.parsed.view_information(self.platform),
            Tab::Logs => RenderedLogSection {
                title: tab.to_string(),
                subsections: self.parsed.view_logs(query),
                ..Default::default()
            }
            .view(self.platform.is_android(), self.platform.is_android(), true),
            Tab::Raw => html! {
                <CodeBlock text=self.text.clone()/>
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
