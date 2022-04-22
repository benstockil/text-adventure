#![warn(clippy::all, clippy::pedantic)]

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    io,
    iter,
    time::Duration,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use text_adventure::{parser::story_parser, StoryEvent};

enum InputMode {
    Disabled,
    Input,
    Pause,
}

enum UpdateState {
    Update,
    Wait,
    HandleInput,
    Responding,
}

struct App {
    story: VecDeque<StoryEvent>,
    input: String,
    input_mode: InputMode,
    output: Vec<String>,
    current_response: String,
    response_progress: usize,
    game_store: HashMap<String, String>,
    update: UpdateState,
    label: String,
}


impl Default for App {
    fn default() -> App {
        let text = include_str!("../story/entry.story");
        let story = story_parser::story(text).unwrap();

        App {
            story: VecDeque::from(story),
            input: String::new(),
            output: Vec::new(),
            response_progress: 0,
            current_response: String::new(),
            input_mode: InputMode::Disabled,
            game_store: HashMap::new(),
            update: UpdateState::Update,
            label: String::default(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        match app.update {
            UpdateState::Update => match app.story.get(app.position) {
                Some(event) => {
                    match event {
                        StoryEvent::Text(t) => {
                            app.current_response = t.clone();
                            app.input_mode = InputMode::Disabled;
                            app.update = UpdateState::Responding;
                        }
                        StoryEvent::Input(_) => {
                            app.input_mode = InputMode::Input;
                            app.update = UpdateState::Wait;
                        }
                        StoryEvent::Pause => {
                            app.input_mode = InputMode::Pause;
                            app.update = UpdateState::Wait;
                        }
                        StoryEvent::Clear => {
                            app.output.clear();
                        }
                    }
                    app.position += 1;
                }
                None => {
                    return Ok(());
                }
            },

            UpdateState::HandleInput => {
                app.game_store
                    .insert(app.label.clone(), app.input.drain(..).collect());
                app.update = UpdateState::Update;
            }

            UpdateState::Responding => {
                if app.response_progress == app.current_response.len() {
                    app.output.push(app.current_response.clone());
                    app.response_progress = 0;
                    app.update = UpdateState::Update;
                } else {
                    // FIX: No time control for responses; one char every frame
                    app.response_progress += 1;
                }
            }

            UpdateState::Wait => {}
        }

        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Home = key.code {
                    return Ok(());
                }

                match app.input_mode {
                    InputMode::Input => match key.code {
                        KeyCode::Enter => {
                            app.input_mode = InputMode::Disabled;
                            app.update = UpdateState::HandleInput;
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        _ => {}
                    },
                    InputMode::Disabled => match key.code {
                        _ => {}
                    },
                    InputMode::Pause => {
                        app.update = UpdateState::Update;
                    }
                }
            }
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
        .split(f.size());

    let text = vec![Spans::from(vec![Span::raw("> "), Span::raw(&app.input)])];
    let paragraph = Paragraph::new(text)
        .style(match app.input_mode {
            InputMode::Disabled => Style::default(),
            InputMode::Input => Style::default().fg(Color::Yellow),
            InputMode::Pause => Style::default().fg(Color::Green),
        })
        .block(Block::default());
    f.render_widget(paragraph, chunks[1]);
    match app.input_mode {
        InputMode::Input => f.set_cursor(chunks[1].x + 2 + app.input.width() as u16, chunks[1].y),
        _ => {}
    }


    let current = app.current_response[0..app.response_progress].to_owned();
    let output = List::new(
        app.output
            .iter()
            .chain(iter::once(&current))
            .map(|t| ListItem::new(t.as_ref()))
            .collect::<Vec<_>>(),
    );

    f.render_widget(output, chunks[0]);
}
