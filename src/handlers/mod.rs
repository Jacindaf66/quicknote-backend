pub mod auth;

pub use auth::{register, login};

pub mod notes;

pub use notes::{create_note, list_notes, get_note, update_note, delete_note};
