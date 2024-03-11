use crate::{days_ago, tag_csv, task::TodoTasks};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::*,
};
use miette::*;
use ratatui::{
    prelude::*,
    widgets::{Cell, Row, Table, TableState},
};
use std::io::{self, stdout, Stdout};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

fn term_init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn term_restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[derive(Copy, Clone, PartialEq)]
enum Exit {
    Continue,
    Save,
    Forget,
}

pub struct Move<'a> {
    pub tasks: &'a mut TodoTasks,
    table_state: TableState,
    exit: Exit,
}

impl<'a> Move<'a> {
    pub fn new(tasks: &'a mut TodoTasks) -> Self {
        Move {
            tasks,
            table_state: TableState::default().with_selected(0),
            exit: Exit::Continue,
        }
    }

    pub fn run(mut self) -> Result<bool> {
        let mut term = term_init().into_diagnostic()?;
        let res = self.run_loop(&mut term);
        term_restore().into_diagnostic()?;
        res.map(|_| match self.exit {
            Exit::Continue | Exit::Save => true,
            Exit::Forget => false,
        })
        .into_diagnostic()
    }

    fn run_loop(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while self.exit == Exit::Continue {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        let table = Table::default()
            .header(
                Row::new(
                    ["Task#", "Description", "Note", "Created", "Tags"]
                        .map(|t| Text::from(t).centered())
                        .map(Cell::from)
                        .to_vec(),
                )
                .style(Style::new().bold()),
            )
            .widths(vec![
                Constraint::Length(5),
                Constraint::Percentage(35),
                Constraint::Percentage(35),
                Constraint::Length(10),
                Constraint::Fill(1),
            ])
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">>")
            .rows(
                self.tasks
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        let mut row = Row::new(vec![
                            Cell::from(Text::from(format!("{}", i + 1)).right_aligned()),
                            Text::from(t.description.clone()).bold().into(),
                            Text::from(t.note.clone()).italic().into(),
                            Text::from(days_ago(t.duration_since_creation()))
                                .centered()
                                .into(),
                            tag_csv(t.tags()).into(),
                        ]);
                        if i % 2 == 1 {
                            row = row.style(Style::new().bg(Color::DarkGray));
                        }
                        row
                    })
                    .collect::<Vec<_>>(),
            );

        let mut size = frame.size();
        size.height = size.height.saturating_sub(1);
        frame.render_stateful_widget(table, size, &mut self.table_state);
        let instructions = "⬆/⬇ Select Row  \
                            +/- Change Priority  \
                            1-6 Set Priority  \
                            D Remove Task  \
                            q Save and exit  \
                            x Exit";
        let instructions = Text::from(instructions).centered();
        let size = Rect {
            y: size.height,
            height: 1,
            ..size
        };
        frame.render_widget(instructions, size)
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let tlen = self.tasks.len();
        let key_ev = match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => Some(key_event),
            _ => None,
        };
        if let Some(key_ev) = key_ev {
            match key_ev.code {
                KeyCode::Char('q') => self.exit = Exit::Save,
                KeyCode::Char('x') => self.exit = Exit::Forget,
                KeyCode::Up => {
                    *self.table_state.selected_mut() = self
                        .table_state
                        .selected()
                        .unwrap_or_default()
                        .saturating_sub(1)
                        .into()
                }
                KeyCode::Down => {
                    *self.table_state.selected_mut() =
                        (self.table_state.selected().unwrap_or_default() + 1)
                            .min(tlen)
                            .into()
                }
                KeyCode::Char('=') => self.move_(|i| i.saturating_sub(1)),
                KeyCode::Char('-') => self.move_(|i| (i + 2).min(tlen)),
                KeyCode::Char('1') => self.move_(|_| 0),
                KeyCode::Char('2') => self.move_(|_| 1.min(tlen)),
                KeyCode::Char('3') => self.move_(|_| 2.min(tlen)),
                KeyCode::Char('4') => self.move_(|_| 3.min(tlen)),
                KeyCode::Char('5') => self.move_(|_| 4.min(tlen)),
                KeyCode::Char('6') => self.move_(|_| 5.min(tlen)),
                KeyCode::Char('D') => {
                    if let Some(i) = self.table_state.selected() {
                        self.tasks.remove(i);
                        *self.table_state.selected_mut() = Some(i.saturating_sub(0));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn move_(&mut self, before: impl FnOnce(usize) -> usize) {
        if let Some(i) = self.table_state.selected() {
            let mut before = before(i);
            if i < before {
                before = before.saturating_sub(1);
            }
            let t = self.tasks.remove(i);
            self.tasks.insert(before, t);
            *self.table_state.selected_mut() = Some(before);
        }
    }
}
