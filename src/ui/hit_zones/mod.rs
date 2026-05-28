#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageAction {
    Previous,
    Next,
}

mod assign;
mod log;
mod menu;
mod toolbar;
mod top_bar;

pub use assign::*;
pub use log::*;
pub use menu::*;
pub use toolbar::*;
pub use top_bar::*;
