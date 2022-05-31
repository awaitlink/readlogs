use std::{
    collections::BTreeMap,
    io::{self, Cursor},
    rc::Rc,
};

use anyhow::{anyhow, ensure, Context};
use derive_more::{Display, IsVariant};
use strum_macros::EnumIter;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewtil::NeqAssign;
use zip::ZipArchive;

use crate::{
    parsers::{AppId, LogFilename},
    *,
};

#[derive(Debug)]
pub enum Msg {
    UpdateUrl(String),
    Start,
    FetchError(anyhow::Error),
    FinishedFetchText(String),
    FinishedFetchBinary(Vec<u8>),
    UpdateActiveFile(Rc<LogFilename>),
    UpdateTab(Tab),
    UpdateMinLogLevel(String),
    UpdateQuery(String),
    UpdateUiExpanded,
    ApplySearchQuery,
}

#[derive(Debug)]
pub enum Object {
    Single(File),
    Multiple {
        files: BTreeMap<Rc<LogFilename>, File>,
        active_filename: Rc<LogFilename>,
    },
}

#[derive(Debug, IsVariant)]
pub enum State {
    NoData,
    Error(anyhow::Error),
    Fetching,
    Ready(Object),
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (State::NoData, State::NoData) => true,
            (State::Error(_), State::Error(_)) => false,
            (State::Fetching, State::Fetching) => true,
            (State::Ready(_), State::Ready(_)) => false,
            _ => false,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::NoData
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SearchQuery {
    pub min_log_level: LogLevel,
    pub string: String,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            min_log_level: LogLevel::Error,
            string: Default::default(),
        }
    }
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, EnumIter, IsVariant)]
pub enum Tab {
    Information,
    Logs,
    Raw,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Information
    }
}

impl Tab {
    pub fn icon(&self) -> Classes {
        match self {
            Tab::Information => classes!("fas", "fa-info"),
            Tab::Logs => classes!("fas", "fa-th-list"),
            Tab::Raw => classes!("fas", "fa-file"),
        }
    }
}

#[derive(Debug)]
pub struct Model {
    pub state: State,
    pub debug_log_input: NodeRef,
    pub debug_log_url: String,
    pub remote_object: Option<RemoteObject>,
    pub tab: Tab,
    pub pending_query: SearchQuery,
    pub active_query: SearchQuery,
    pub ui_expanded: bool,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &yew::prelude::Context<Self>) -> Self {
        Self {
            state: Default::default(),
            debug_log_input: NodeRef::default(),
            debug_log_url: Default::default(),
            remote_object: None,
            tab: Default::default(),
            pending_query: Default::default(),
            active_query: Default::default(),
            ui_expanded: false,
        }
    }

    fn update(&mut self, ctx: &yew::prelude::Context<Self>, msg: Self::Message) -> bool {
        match self.update_inner(ctx, msg) {
            Ok(should_render) => should_render,
            Err(e) => self.state.neq_assign(State::Error(e)),
        }
    }

    fn view(&self, ctx: &yew::prelude::Context<Self>) -> Html {
        self.view_inner(ctx)
    }
}

impl Model {
    fn fetch(&self, ctx: &yew::prelude::Context<Self>, url: &str, binary: bool) {
        let request = reqwasm::http::Request::get(url);

        ctx.link().send_future(async move {
            match request.send().await {
                Ok(response) => match response.status() {
                    200 => {
                        if binary {
                            let bytes = response.binary().await.unwrap();
                            Msg::FinishedFetchBinary(bytes)
                        } else {
                            let text = response.text().await.unwrap();
                            Msg::FinishedFetchText(text)
                        }
                    }
                    _ => Msg::FetchError(
                        anyhow!("status code {}", response.status())
                            .context("fetching debug log finished unsuccessfully"),
                    ),
                },
                Err(e) => Msg::FetchError(
                    anyhow::Error::from(e).context("couldn't start fetching debug log"),
                ),
            }
        })
    }

    pub(super) fn active_file(&self) -> &File {
        match &self.state {
            State::Ready(Object::Single(file)) => file,
            State::Ready(Object::Multiple {
                files,
                active_filename,
            }) => files.get(active_filename).unwrap(),
            _ => panic!("State is not `Ready`"),
        }
    }

    fn update_inner(
        &mut self,
        ctx: &yew::prelude::Context<Self>,
        msg: <Self as Component>::Message,
    ) -> anyhow::Result<bool> {
        match msg {
            Msg::UpdateUrl(value) => Ok(self.debug_log_url.neq_assign(value)),
            Msg::Start => match &self.state {
                State::NoData | State::Error(_) | State::Ready(_) => {
                    if let Some(input) = self.debug_log_input.cast::<HtmlInputElement>() {
                        let _ = input.blur();
                    }

                    self.tab = Default::default();
                    self.pending_query = Default::default();
                    self.active_query = Default::default();

                    let reference = self
                        .debug_log_url
                        .trim()
                        .to_lowercase()
                        .parse::<RemoteObject>()
                        .context("failed to parse the debug log URL")?;

                    self.fetch(
                        ctx,
                        &reference.fetchable_url(),
                        matches!(reference.platform(), Platform::Ios),
                    );

                    self.debug_log_url = reference.debuglogs_url();
                    self.remote_object = Some(reference);

                    Ok(self.state.neq_assign(State::Fetching))
                }
                _ => Ok(false),
            },
            Msg::FetchError(e) => Err(e),
            Msg::FinishedFetchText(text) => {
                let file = File::from_text(self.remote_object.clone().unwrap(), None, text);

                Ok(self.state.neq_assign(State::Ready(Object::Single(file))))
            }
            Msg::FinishedFetchBinary(bytes) => {
                let mut zip = ZipArchive::new(Cursor::new(bytes.as_slice()))
                    .context("couldn't read the debug log file as a `zip`")?;

                let mut files = BTreeMap::new();

                for i in 0..zip.len() {
                    let mut file = zip.by_index(i)?;

                    let name = Rc::new(
                        file.name()
                            .parse::<LogFilename>()
                            .context("couldn't parse a file's name")?,
                    );

                    let mut bytes: Vec<u8> = vec![];
                    io::copy(&mut file, &mut bytes)
                        .context("couldn't copy a log file into a `Vec<u8>`")?;
                    let text = String::from_utf8(bytes)
                        .context("couldn't turn a `Vec<u8>` into a `String`")?;

                    files.insert(
                        Rc::clone(&name),
                        File::from_text(self.remote_object.clone().unwrap(), Some(name), text),
                    );
                }

                ensure!(!files.is_empty(), "no files in zip"); // TODO: maybe should just be a notice instead of an error

                let last_for_app_id = |app_id| files.keys().filter(|k| k.app_id == app_id).last();
                let active_filename =
                    Rc::clone(last_for_app_id(AppId::Signal).unwrap_or_else(|| {
                        last_for_app_id(AppId::NotificationServiceExtension)
                            .unwrap_or_else(|| last_for_app_id(AppId::ShareAppExtension).unwrap())
                    }));

                Ok(self.state.neq_assign(State::Ready(Object::Multiple {
                    files,
                    active_filename,
                })))
            }
            Msg::UpdateActiveFile(filename) => Ok(
                if let State::Ready(Object::Multiple {
                    active_filename, ..
                }) = &mut self.state
                {
                    active_filename.neq_assign(filename)
                } else {
                    false
                },
            ),
            Msg::UpdateTab(tab) => Ok(self.tab.neq_assign(tab)),
            Msg::UpdateMinLogLevel(value) => Ok(self
                .pending_query
                .min_log_level
                .neq_assign(value.parse().unwrap())),
            Msg::UpdateQuery(value) => Ok(self.pending_query.string.neq_assign(value)),
            Msg::UpdateUiExpanded => {
                self.ui_expanded = !self.ui_expanded;
                Ok(true)
            }
            Msg::ApplySearchQuery => Ok(self.active_query.neq_assign(self.pending_query.clone())),
        }
    }
}
