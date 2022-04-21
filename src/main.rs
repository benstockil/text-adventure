use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{self, Read},
    time::Duration,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

mod parser;
use parser::story_parser;

enum InputMode {
    Disabled,
    Input,
    Pause,
}

#[derive(Debug)]
pub enum StoryEvent {
    Text(String),
    Input(String),
    Pause,
    Clear,
}

enum UpdateState {
    Update,
    Wait,
    Handle,
}

struct App {
    story: Vec<StoryEvent>,
    position: usize,
    input: String,
    input_mode: InputMode,
    output: Vec<String>,
    game_store: HashMap<String, String>,
    update: UpdateState,
    label: String,
}

impl Default for App {
    fn default() -> App {
        let text = include_str!("../story/entry.story");
        let story = story_parser::story(&text).unwrap();

        App {
            story,
            position: 0,
            input: String::new(),
            output: Vec::new(),
            input_mode: InputMode::Disabled,
            game_store: HashMap::new(),
            update: UpdateState::Update,
            label: "".into(),
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
        println!("{:?}", err)
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
                            app.output.push(t.clone());
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

            UpdateState::Handle => {
                app.game_store
                    .insert(app.label.clone(), app.input.drain(..).collect());
                app.update = UpdateState::Update;
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
                            app.update = UpdateState::Handle;
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

    let messages: Vec<ListItem> = app
        .output
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
            ListItem::new(content)
        })
        .collect();
    let messages = List::new(messages).block(Block::default());
    f.render_widget(messages, chunks[0]);
}
