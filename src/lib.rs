#![warn(clippy::all, clippy::pedantic)]

use crate::parser::story_parser::story;
use std::collections::{HashMap, VecDeque};

pub mod parser;

#[derive(Debug)]
pub enum StoryEvent {
    Text(String),
    Input(String),
    Pause,
    Clear,
}

pub struct AppData {
    pub story: VecDeque<StoryEvent>,
    pub game_store: HashMap<String, String>,
}

impl Default for AppData {
    fn default() -> AppData {
        let text = include_str!("../story/entry.story");
        let story = story(text).unwrap();

        AppData {
            story: VecDeque::from(story),
            game_store: HashMap::new(),
        }
    }
}
