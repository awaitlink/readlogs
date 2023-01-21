use std::rc::Rc;

use yew::prelude::*;

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

#[function_component(FilePicker)]
pub fn file_picker(props: &FilePickerProps) -> Html {
    let submission_time = props.files[0].submission_time;

    html! {
        <Message
            classes={props.classes.clone()}
            heading={format!("{submission_time} AM/PM")}
        >
            <Table classes={classes!("font-mono")}>
                <tbody>
                    { for props.files.iter().map(|file| view_file_row(props, Rc::clone(file))) }
                </tbody>
            </Table>
        </Message>
    }
}

fn view_file_row(props: &FilePickerProps, file: Rc<LogFilename>) -> Html {
    let active = props.selected_file == file;
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
            {classes}
            on_click={props.on_file_selected.clone().reform(move |_| Rc::clone(&file))}
        >
            <TableItem><Icon icon={classes!("fas", icon)} /></TableItem>
            <TableItem>{ app_id }</TableItem>
            <TableItem>{ file_time }</TableItem>
        </TableRow>
    }
}
