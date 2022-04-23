#![warn(clippy::all, clippy::pedantic)]

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
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

use text_adventure::{StoryEvent, AppData};

enum InputMode {
    Disabled,
    Input,
    Pause,
}
impl Default for InputMode {
    fn default() -> Self {
        Self::Disabled
    }
}

enum UpdateState {
    Update,
    Wait,
    HandleInput,
    Responding,
}
impl Default for UpdateState {
    fn default() -> Self {
        Self::Update
    }
}

#[derive(Default)]
struct AppUI {
    input: String,
    input_mode: InputMode,
    output: Vec<String>,
    current_response: String,
    response_progress: usize,
    update: UpdateState,
    label: String,
}


fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app_data = AppData::default();
    let app_ui = AppUI::default();
    let res = run_app(&mut terminal, app_ui, app_data);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app_ui: AppUI, mut app_data: AppData) -> io::Result<()> {
    loop {
        match app_ui.update {
            UpdateState::Update => match app_data.story.pop_front() {
                Some(event) => {
                    match event {
                        StoryEvent::Text(t) => {
                            app_ui.current_response = t.clone();
                            app_ui.input_mode = InputMode::Disabled;
                            app_ui.update = UpdateState::Responding;
                        }
                        StoryEvent::Input(_) => {
                            app_ui.input_mode = InputMode::Input;
                            app_ui.update = UpdateState::Wait;
                        }
                        StoryEvent::Pause => {
                            app_ui.input_mode = InputMode::Pause;
                            app_ui.update = UpdateState::Wait;
                        }
                        StoryEvent::Clear => {
                            app_ui.output.clear();
                        }
                    }
                }
                None => {
                    return Ok(());
                }
            },

            UpdateState::HandleInput => {
                app_data.game_store
                    .insert(app_ui.label.clone(), app_ui.input.drain(..).collect());
                app_ui.update = UpdateState::Update;
            }

            UpdateState::Responding => {
                if app_ui.response_progress == app_ui.current_response.len() {
                    app_ui.output.push(app_ui.current_response.clone());
                    app_ui.response_progress = 0;
                    app_ui.update = UpdateState::Update;
                } else {
                    // FIX: No time control for responses; one char every frame
                    app_ui.response_progress += 1;
                }
            }

            UpdateState::Wait => {}
        }

        terminal.draw(|f| ui(f, &app_ui))?;

        if event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Home = key.code {
                    return Ok(());
                }

                match app_ui.input_mode {
                    InputMode::Input => match key.code {
                        KeyCode::Enter => {
                            app_ui.input_mode = InputMode::Disabled;
                            app_ui.update = UpdateState::HandleInput;
                        }
                        KeyCode::Char(c) => {
                            app_ui.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app_ui.input.pop();
                        }
                        _ => {}
                    },
                    InputMode::Disabled => match key.code {
                        _ => {}
                    },
                    InputMode::Pause => {
                        app_ui.update = UpdateState::Update;
                    }
                }
            }
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn ui<B: Backend>(f: &mut Frame<B>, app: &AppUI) {
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