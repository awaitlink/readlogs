use yew::prelude::*;

use crate::components::{Link, Title, TitleLevel};

#[derive(Debug)]
pub struct RenderedLogSection {
    pub level: TitleLevel,
    pub title: String,

    pub displayed_count: usize,
    pub total_count: usize,

    pub html: Html,

    pub subsections: Vec<RenderedLogSection>,
}

impl RenderedLogSection {
    // TODO: better arguments
    pub fn view(
        &self,
        show_table_of_contents: bool,
        show_other_titles: bool,
        show_title: bool,
    ) -> Html {
        let table_of_contents = if show_table_of_contents {
            html! {
                <>
                    { self.table_of_contents() }
                    <hr/>
                </>
            }
        } else {
            html! {}
        };

        let title = if show_title {
            html! { self.title(self.level, Some(self.title_id())) }
        } else {
            html! {}
        };

        html! {
            <>
                { table_of_contents }
                { title }
                { self.html.clone() }
                { for self.subsections.iter().map(|subsection| subsection.view(false, show_other_titles, show_other_titles)) }
            </>
        }
    }

    fn total_displayed_count(&self) -> usize {
        self.displayed_count
            + self
                .subsections
                .iter()
                .fold(0, |acc, e| acc + e.total_displayed_count())
    }

    // TODO: not the best name...
    fn total_total_count(&self) -> usize {
        self.total_count
            + self
                .subsections
                .iter()
                .fold(0, |acc, e| acc + e.total_total_count())
    }

    fn title_id(&self) -> String {
        self.title.to_lowercase().replace(' ', "-")
    }

    fn title(&self, level: TitleLevel, id: Option<String>) -> Html {
        let raw = self.level > TitleLevel::H2;

        html! {
            <Title
                {level}
                text={self.title.clone()}
                meta={format!("{}/{}", self.total_displayed_count(), self.total_total_count())}
                {id}
                {raw}
                capitalize={!raw}
            />
        }
    }

    fn table_of_contents(&self) -> Html {
        html! {
            <ul>
                <li>
                    <Link
                        href={String::from("#") + &self.title_id()}
                        new_tab={false}
                    >
                        { self.title(TitleLevel::H4, None) }
                    </Link>

                    { for self.subsections.iter().map(Self::table_of_contents) }
                </li>
            </ul>
        }
    }
}

impl Default for RenderedLogSection {
    fn default() -> Self {
        RenderedLogSection {
            level: TitleLevel::H1,
            title: Default::default(),
            displayed_count: 0,
            total_count: 0,
            html: html! {},
            subsections: vec![],
        }
    }
}
