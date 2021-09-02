pub mod components;
mod file;
mod log_level;
mod model;
mod parsers;
mod platform;
pub mod post_processing;
mod remote_object;
mod rendered_log_section;
mod utils;
mod view;

pub use file::File;
pub use log_level::LogLevel;
pub use model::*;
pub use platform::Platform;
pub use remote_object::RemoteObject;
pub use rendered_log_section::RenderedLogSection;
pub use utils::*;

fn main() {
    yew::start_app::<Model>();
}
