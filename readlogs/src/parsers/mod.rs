use anyhow::anyhow;
use chrono::prelude::*;
use nom::{combinator::eof, sequence::terminated};
pub use nom_tracable::{tracable_parser as traceable_parser, TracableInfo as TraceableInfo};
pub use readlogs_macros::traceable_configurable_parser;
use yew::prelude::*;

use crate::{
    components::*, span, LogLevel, Platform, RemoteObject, RenderedLogSection, SearchQuery,
};

mod android;
mod common;
mod desktop;
mod ios;
mod ios_filename;

pub use ios_filename::*;

pub type Span<'a> = nom_locate::LocatedSpan<&'a str, TraceableInfo>;

#[derive(Debug, Clone, PartialEq)]
pub struct Content {
    pub information: Vec<Section<InfoEntry>>,
    pub logs: Vec<Section<LogEntry>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section<C> {
    pub name: String,
    pub content: Vec<C>,
    pub subsections: Vec<Section<C>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InfoEntry {
    KeyValue(String, Value),
    KeyEnabledValue(String, bool, Option<Value>),
    RemoteObject(RemoteObject),
    ExplicitNone,
    GenericTable(GenericTable),
    Generic(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Generic(String),
    BucketedFlag(Vec<Bucket>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericTable {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Default for Value {
    fn default() -> Self {
        Value::Generic(Default::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bucket {
    pub country_code: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: Option<LogLevel>,
    pub meta: PlatformMetadata, // TODO: don't repeat in every log message if there is no real metadata?
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformMetadata {
    AndroidLogcat {
        process_id: String,
        thread_id: String,
        tag: String,
    },
    AndroidLogger {
        version: String,
        thread_id: String,
        tag: String,
    },
    Ios(Option<ios::LogEntryMetadata>),
    Desktop,
}

impl Content {
    pub fn parse(platform: Platform, text: &str) -> anyhow::Result<Self> {
        let parsing_fn = match platform {
            Platform::Android => android::content,
            Platform::Ios => ios::content,
            Platform::Desktop => desktop::content,
        };

        let (_, output) =
            terminated(parsing_fn, eof)(span(text)).map_err(|error| anyhow!("{:#?}", error))?;

        Ok(output)
    }

    pub fn view_information(&self, platform: Platform) -> Html {
        if self.information.is_empty() {
            let text = match platform {
                Platform::Ios => {
                    "Signal iOS debug logs don't contain any dedicated information sections."
                }
                Platform::Android | Platform::Desktop => {
                    "This file doesn't seem to contain any dedicated information sections."
                }
            };

            html! {
                <Message {text} />
            }
        } else {
            html! { for self.information.iter().map(|section| section.view(TitleLevel::H2)) }
        }
    }

    pub fn view_logs(&self, query: &SearchQuery) -> Vec<RenderedLogSection> {
        self.logs
            .iter()
            .map(|section| section.view(query, TitleLevel::H2))
            .collect()
    }
}

impl Section<InfoEntry> {
    pub fn view(&self, level: TitleLevel) -> Html {
        let content = html! { for self.content.iter().map(|entry| entry.view()) };

        // TODO: Assumes that all entries in the section are the same variant.
        let wrapper = match self.content.first() {
            Some(InfoEntry::KeyValue(_, _)) | Some(InfoEntry::KeyEnabledValue(_, _, _)) => html! {
                <Table classes={classes!("font-mono", "text-sm")}>
                    <tbody>
                        { content }
                    </tbody>
                </Table>
            },
            Some(InfoEntry::Generic(_)) => html! {
                <CodeBlock>
                    { content }
                </CodeBlock>
            },
            _ => content,
        };

        let full_content = if self.content.is_empty() && self.subsections.is_empty() {
            html! {
                <p><i>{ "No content or subsections" }</i></p>
            }
        } else {
            html! {
                <>
                    { wrapper }
                    { for self.subsections.iter().map(|section| section.view(level.incremented().unwrap())) }
                </>
            }
        };

        let raw = level > TitleLevel::H2;

        html! {
            <>
                <Title {level} text={self.name.clone()} {raw} capitalize={!raw} />
                { full_content }
            </>
        }
    }
}

impl InfoEntry {
    pub fn view(&self) -> Html {
        match self {
            InfoEntry::KeyValue(key, value) => html! {
                <TableRow>
                    <TableItem>{ key }</TableItem>
                    <TableItem>{ value.view() }</TableItem>
                </TableRow>
            },
            InfoEntry::KeyEnabledValue(key, enabled, value) => html! {
                <TableRow>
                    <TableItem>{ key }</TableItem>
                    {
                        // TODO: Assumes that all KV pairs in section have `enabled`
                        //       (otherwise the table will be misaligned).
                        html! {
                            <TableItem>
                                {
                                    if *enabled {
                                        "enabled"
                                    } else {
                                        "disabled"
                                    }
                                }
                            </TableItem>
                        }
                    }
                    <TableItem>{ value.clone().unwrap_or_default().view() }</TableItem>
                </TableRow>
            },
            InfoEntry::RemoteObject(ro) => html! {
                <Button
                    size={ButtonSize::Medium}
                    icon={classes!("fas", "fa-download")}
                    text="debuglogs.org"
                    href={ro.debuglogs_url()}
                />
            },
            InfoEntry::ExplicitNone => html! { <p>{ "None" }</p> },
            InfoEntry::GenericTable(table) => html! {
                <Table>
                    <thead>
                        <TableRow>
                            {
                                for table.header.iter().map(|item| html! {
                                    <TableItem>{ item }</TableItem>
                                })
                            }
                        </TableRow>
                    </thead>
                    <tbody>
                        {
                            for table.rows.iter().map(|row| html! {
                                <TableRow>
                                    {
                                        for row.iter().map(|item| html! {
                                            <TableItem>{ item }</TableItem>
                                        })
                                    }
                                </TableRow>
                            })
                        }
                    </tbody>
                </Table>
            },
            InfoEntry::Generic(text) => html! { text.to_owned() + "\n" },
        }
    }
}

impl Value {
    pub fn view(&self) -> Html {
        match self {
            Value::Generic(s) => html! { s },
            Value::BucketedFlag(buckets) => html! {
                <Table>
                    <thead>
                        <TableRow>
                            <TableItem tag="th" classes={classes!("!border-r")}>{ "Country code" }</TableItem>
                            {
                                for buckets.iter().map(|bucket| html! {
                                    <TableItem>{ &bucket.country_code }</TableItem>
                                })
                            }
                        </TableRow>
                    </thead>
                    <tbody>
                        <TableRow>
                            <TableItem tag="th" classes={classes!("!border-r")}>{ "Value" }</TableItem>
                            {
                                for buckets.iter().map(|bucket| html! {
                                    <TableItem>{ &bucket.value }</TableItem>
                                })
                            }
                        </TableRow>
                    </tbody>
                </Table>
            },
        }
    }
}

impl Section<LogEntry> {
    pub fn view(&self, query: &SearchQuery, level: TitleLevel) -> RenderedLogSection {
        let s = &query.string.to_lowercase();
        let entries_to_display = self
            .content
            .iter()
            .enumerate()
            .map(|(index, entry)| (index + 1, entry))
            .filter(|(_, entry)| entry.level.unwrap_or_default() >= query.min_log_level)
            .filter(|(_, entry)| {
                entry.timestamp.to_lowercase().contains(s)
                    || entry.message.to_lowercase().contains(s)
                    || entry.meta.contains(s)
            });

        let displayed_count = entries_to_display.clone().count();
        let total_count = self.content.len();

        let table = if displayed_count != 0 {
            html! {
                <Table>
                    <thead>
                        <TableRow classes={classes!("text-left")}>
                            <TableItem tag="th"><Icon fixed_width_height=false icon={classes!("fas", "fa-hashtag")}/></TableItem>
                            <TableItem tag="th" classes={classes!("min-w-[250px]")}>{ "Timestamp" }</TableItem>

                            {
                                match &self.content.get(0).unwrap().meta { // TODO: assumption?
                                    PlatformMetadata::AndroidLogcat { .. } => html! {
                                        <>
                                            <TableItem tag="th">{ "Process" }</TableItem>
                                            <TableItem tag="th">{ "Thread" }</TableItem>
                                            <TableItem tag="th">{ "Tag" }</TableItem>
                                        </>
                                    },
                                    PlatformMetadata::AndroidLogger { .. } => html! {
                                        <>
                                            <TableItem tag="th">{ "Version" }</TableItem>
                                            <TableItem tag="th">{ "Thread" }</TableItem>
                                            <TableItem tag="th">{ "Tag" }</TableItem>
                                        </>
                                    },
                                    PlatformMetadata::Ios(_) => html! {
                                        <>
                                            <TableItem tag="th">{ "File" }</TableItem>
                                            <TableItem tag="th">{ "Line" }</TableItem>
                                            <TableItem tag="th">{ "Symbol" }</TableItem>
                                        </>
                                    },
                                    PlatformMetadata::Desktop => html! {},
                                }
                            }

                            <TableItem tag="th">{ "Message" }</TableItem>
                        </TableRow>
                    </thead>
                    <tbody class="font-mono">
                        { for entries_to_display.map(|(number, entry)| entry.view(number)) }
                    </tbody>
                </Table>
            }
        } else {
            html! {}
        };

        let subsections = self
            .subsections
            .iter()
            .map(|subsection| subsection.view(query, level.incremented().unwrap()))
            .collect();

        RenderedLogSection {
            level,
            title: self.name.clone(),
            displayed_count,
            total_count,
            html: table,
            subsections,
        }
    }
}

impl LogEntry {
    pub fn view(&self, number: usize) -> Html {
        html! {
            <TableRow classes={self.level.unwrap_or_default().color()}>
                <TableItem>{ number }</TableItem>
                <TableItem>{ self.timestamp.to_string() }</TableItem>
                { self.meta.clone().view() }
                <TableItem><pre>{ self.message.to_owned() }</pre></TableItem>
            </TableRow>
        }
    }
}

impl PlatformMetadata {
    pub fn contains(&self, s: &str) -> bool {
        match &self {
            PlatformMetadata::AndroidLogcat {
                process_id,
                thread_id,
                tag,
            } => process_id.contains(s) || thread_id.contains(s) || tag.contains(s),
            PlatformMetadata::AndroidLogger {
                version,
                thread_id,
                tag,
            } => version.contains(s) || thread_id.contains(s) || tag.contains(s),
            PlatformMetadata::Ios(Some(meta)) => {
                meta.file.to_lowercase().contains(s)
                    || meta.line.to_lowercase().contains(s)
                    || meta.symbol.to_lowercase().contains(s)
            }
            PlatformMetadata::Ios(None) | PlatformMetadata::Desktop => false,
        }
    }

    pub fn view(self) -> Html {
        match self {
            PlatformMetadata::AndroidLogcat {
                process_id,
                thread_id,
                tag,
            } => html! {
                <>
                    <TableItem>{ process_id }</TableItem>
                    <TableItem>{ thread_id }</TableItem>
                    <TableItem>{ tag }</TableItem>
                </>
            },
            PlatformMetadata::AndroidLogger {
                version,
                thread_id,
                tag,
            } => html! {
                <>
                    <TableItem>{ version }</TableItem>
                    <TableItem>{ thread_id }</TableItem>
                    <TableItem>{ tag }</TableItem>
                </>
            },
            PlatformMetadata::Ios(Some(meta)) => html! {
                <>
                    <TableItem>{ meta.file }</TableItem>
                    <TableItem>{ meta.line }</TableItem>
                    <TableItem classes={classes!("text-right")}><pre>{ meta.symbol }</pre></TableItem>
                </>
            },
            PlatformMetadata::Ios(None) => html! {
                <>
                    <TableItem/>
                    <TableItem/>
                    <TableItem/>
                </>
            },
            PlatformMetadata::Desktop => html! {},
        }
    }
}
