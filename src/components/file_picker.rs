use std::rc::Rc;

use yew::prelude::*;
use yewtil::NeqAssign;

use crate::{
    components::{Icon, Message, Table, TableItem, TableRow},
    parsers::{AppId, LogFilename},
};

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct FilePickerProps {
    #[prop_or_default]
    pub classes: Classes,

    pub files: Vec<Rc<LogFilename>>,
    pub selected_file: Rc<LogFilename>,
    pub on_file_selected: Callback<Rc<LogFilename>>,
}

#[derive(Debug)]
pub struct FilePicker {
    link: ComponentLink<Self>,
    props: FilePickerProps,
}

impl Component for FilePicker {
    type Message = Rc<LogFilename>;
    type Properties = FilePickerProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.props.on_file_selected.emit(msg);
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let submission_time = self.props.files[0].submission_time;

        html! {
            <Message
                classes=self.props.classes.clone()
                heading=format!("{} AM/PM", submission_time)
            >
                <Table classes=classes!("font-mono")>
                    <tbody>
                        { for self.props.files.iter().map(|file| self.view_file_row(Rc::clone(file))) }
                    </tbody>
                </Table>
            </Message>
        }
    }
}

impl FilePicker {
    fn view_file_row(&self, file: Rc<LogFilename>) -> Html {
        let active = self.props.selected_file == file;
        let app_id = file.app_id;
        let file_time = file.file_time.to_string();

        let icon = match app_id {
            AppId::Signal => "fa-square",
            AppId::NotificationServiceExtension => "fa-bell",
            AppId::ShareAppExtension => "fa-share",
        };

        let mut classes = classes!(
            "cursor-pointer",
            "hover:bg-brand-primary-hover",
            "dark:hover:bg-brand-dark-primary-hover",
            "hover:text-brand-text-primary-hover",
            "dark:hover:text-brand-dark-text-primary-hover"
        );

        classes.push(if active {
            classes!(
                "bg-brand-primary-active",
                "dark:bg-brand-dark-primary-active",
                "text-brand-text-primary-active",
                "dark:text-brand-dark-text-primary-active"
            )
        } else {
            classes!()
        });

        html! {
            <TableRow
                classes=classes
                on_click=self.link.callback(move |_| Rc::clone(&file))
            >
                <TableItem><Icon icon=classes!("fas", icon) /></TableItem>
                <TableItem>{ app_id }</TableItem>
                <TableItem>{ file_time }</TableItem>
            </TableRow>
        }
    }
}
