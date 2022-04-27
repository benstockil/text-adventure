#![warn(clippy::all, clippy::pedantic)]

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, iter, mem, time::{Duration, Instant}};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use text_adventure::{AppData, StoryEvent};

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

struct AppUi {
    input: String,
    input_mode: InputMode,
    output: Vec<String>,
    current_response: String,
    response_time: Instant,
    response_progress: usize,
    update: UpdateState,
    label: String,
}

impl Default for AppUi {
    fn default() -> Self {
        Self {
            input: String::default(),
            input_mode: InputMode::default(),
            output: Vec::default(),
            current_response: String::default(),
            // HACK: Unnecessary system call
            response_time: Instant::now(),
            response_progress: usize::default(),
            update: UpdateState::default(),
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
    let app_data = AppData::default();
    let app_ui = AppUi::default();
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

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app_ui: AppUi,
    mut app_data: AppData,
) -> io::Result<()> {
    loop {
        match app_ui.update {
            UpdateState::Update => match app_data.story.pop_front() {
                Some(event) => match event {
                    StoryEvent::Text(t) => {
                        app_ui.current_response = t;
                        app_ui.response_time = Instant::now();
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
                },
                None => {
                    return Ok(());
                }
            },

            UpdateState::HandleInput => {
                app_data
                    .game_store
                    .insert(
                        mem::take(&mut app_ui.label),
                        mem::take(&mut app_ui.input),
                    );
                app_ui.update = UpdateState::Update;
            }

            UpdateState::Responding => {
                let progress = app_ui.response_time.elapsed().as_millis() as usize / 10;
                if progress >= app_ui.current_response.len() {
                    app_ui.output.push(mem::take(&mut app_ui.current_response));
                    app_ui.update = UpdateState::Update;
                    app_ui.response_progress = 0;
                } else {
                    app_ui.response_progress = progress;
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
fn ui<B: Backend>(f: &mut Frame<B>, app: &AppUi) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
        .split(f.size());

    let (content, style) = match app.input_mode {
        InputMode::Disabled => (
            Span::raw("..."),
            Style::default(),
        ),
        InputMode::Input => (
            Span::raw(&app.input),
            Style::default().fg(Color::Yellow),
        ),
        InputMode::Pause => (
            Span::raw("Press any key to continue."),
            Style::default().fg(Color::Green),
        ),
    };
    let text = Spans::from(vec![Span::raw("> "), content]);
    let paragraph = Paragraph::new(text)
        .style(style)
        .block(Block::default());
    f.render_widget(paragraph, chunks[1]);

    match app.input_mode {
        InputMode::Input => f.set_cursor(chunks[1].x + 2 + app.input.width() as u16, chunks[1].y),
        _ => {}
    }

    let current = &app.current_response[0..app.response_progress];
    let output = List::new(
        app.output
            .iter()
            .map(|t| ListItem::new(t.as_str()))
            .chain(iter::once(ListItem::new(current)))
            .collect::<Vec<_>>(),
    );

    f.render_widget(output, chunks[0]);
}
