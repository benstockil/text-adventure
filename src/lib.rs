#![warn(clippy::all, clippy::pedantic)]

pub mod parser;

#[derive(Debug)]
pub enum StoryEvent {
    Text(String),
    Input(String),
    Pause,
    Clear,
}
