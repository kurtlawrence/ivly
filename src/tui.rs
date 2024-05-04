use crate::{
    days_ago, tag_csv,
    task::{TodoTask, TodoTasks},
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::*,
};
use miette::*;
use ratatui::{
    prelude::*,
    widgets::{Block, Cell, Row, Table, TableState},
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

enum Editing {
    Desc { idx: usize, val: String },
    Note { idx: usize, val: String },
    Tags { idx: usize, val: String },
    None,
}

impl Editing {
    fn is_editing(&self) -> bool {
        !matches!(self, Editing::None)
    }

    fn take(&mut self) -> Self {
        std::mem::replace(self, Self::None)
    }

    fn val(&mut self) -> Option<&mut String> {
        match self {
            Self::Desc { val, .. } | Self::Note { val, .. } | Self::Tags { val, .. } => Some(val),
            Self::None => None,
        }
    }

    fn push_char(&mut self, ch: char) {
        if let Some(val) = self.val() {
            val.push(ch);
        }
    }

    fn pop_char(&mut self) {
        if let Some(val) = self.val() {
            val.pop();
        }
    }

    /// If editing this description, creating the 'editing' text.
    fn desc(&self, idx_: usize, task: &TodoTask) -> Text {
        let txt = match self {
            Self::Desc { idx, val } if *idx == idx_ => Text::from(val.clone()).yellow(),
            _ => Text::from(task.description.clone()),
        }
        .bold();
        if task.is_finished() {
            txt.crossed_out()
        } else {
            txt
        }
    }

    /// If editing this note, creating the 'editing' text.
    fn note(&self, idx_: usize, task: &TodoTask) -> Text {
        match self {
            Self::Note { idx, val } if *idx == idx_ => Text::from(val.clone()).italic().yellow(),
            _ => Text::from(task.note.clone()).italic(),
        }
    }

    /// If editing this tags, creating the 'editing' text.
    fn tags(&self, idx_: usize, task: &TodoTask) -> Text {
        match self {
            Self::Tags { idx, val } if *idx == idx_ => Text::from(val.clone()).yellow(),
            _ => Text::from(tag_csv(task.tags())),
        }
    }
}

pub struct Move<'a> {
    pub tasks: &'a mut TodoTasks,
    table_state: TableState,
    exit: Exit,
    show_help: bool,
    editing: Editing,
}

impl<'a> Move<'a> {
    pub fn new(tasks: &'a mut TodoTasks) -> Self {
        Move {
            tasks,
            table_state: TableState::default().with_selected(0),
            exit: Exit::Continue,
            show_help: false,
            editing: Editing::None,
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
        self.render_table(frame);

        let size = frame.size();
        let instructions = if self.editing.is_editing() {
            "Enter to accept changes"
        } else {
            "? Toggle Help  X Exit  q Save and exit"
        };
        let instructions = Text::from(instructions).centered();
        let size = Rect {
            y: size.height.saturating_sub(1),
            height: 1,
            ..size
        };
        frame.render_widget(instructions, size);

        if self.show_help {
            render_help(frame)
        }
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
            if self.editing.is_editing() {
                self.handle_editing(key_ev.code);
            } else {
                match key_ev.code {
                    KeyCode::Char('q') => self.exit = Exit::Save,
                    KeyCode::Char('X') => self.exit = Exit::Forget,
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
                    KeyCode::Home => *self.table_state.selected_mut() = 0.into(),
                    KeyCode::End => {
                        *self.table_state.selected_mut() = tlen.saturating_sub(1).into()
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
                    KeyCode::Char('a') => {
                        let i = self.tasks.len();
                        self.tasks.push(TodoTask::new(""));
                        *self.table_state.selected_mut() = Some(i);
                        self.start_editing_desc()
                    }
                    KeyCode::Char('?') => self.show_help = !self.show_help,
                    KeyCode::Char('e') => self.start_editing_desc(),
                    KeyCode::Char('n') => self.start_editing_note(),
                    KeyCode::Char('t') => self.start_editing_tags(),
                    _ => {}
                }
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

    fn start_editing_desc(&mut self) {
        let idx = self.table_state.selected().unwrap_or_default();
        let val = self
            .tasks
            .get(idx)
            .map(|t| t.description.clone())
            .unwrap_or_default();
        self.editing = Editing::Desc { idx, val };
    }

    fn start_editing_note(&mut self) {
        let idx = self.table_state.selected().unwrap_or_default();
        let val = self
            .tasks
            .get(idx)
            .map(|t| t.note.clone())
            .unwrap_or_default();
        self.editing = Editing::Note { idx, val };
    }

    fn start_editing_tags(&mut self) {
        let idx = self.table_state.selected().unwrap_or_default();
        let val = self
            .tasks
            .get(idx)
            .map(|t| tag_csv(t.tags()))
            .unwrap_or_default();
        self.editing = Editing::Tags { idx, val };
    }

    fn handle_editing(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Enter => match self.editing.take() {
                Editing::None => (),
                Editing::Desc { idx, val } => {
                    if let Some(task) = self.tasks.get_mut(idx) {
                        task.description = val;
                    }
                }
                Editing::Note { idx, val } => {
                    if let Some(task) = self.tasks.get_mut(idx) {
                        task.note = val;
                    }
                }
                Editing::Tags { idx, val } => {
                    if let Some(task) = self.tasks.get_mut(idx) {
                        task.tags = val.split(',').map(String::from).collect();
                    }
                }
            },
            KeyCode::Backspace => {
                self.editing.pop_char();
            }
            KeyCode::Char(c) => {
                self.editing.push_char(c);
            }
            _ => {}
        }
    }

    fn render_table(&mut self, frame: &mut Frame) {
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
                        let desc = self.editing.desc(i, t);
                        let note = self.editing.note(i, t);
                        let tags = self.editing.tags(i, t);
                        let mut row = Row::from_iter([
                            Text::from(format!("{}", i + 1)).right_aligned(),
                            desc,
                            note,
                            Text::from(days_ago(t.duration_since_creation())).centered(),
                            tags,
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
    }
}

fn render_help(frame: &mut Frame) {
    let rows = [
        Row::from_iter([Text::from("⬆/⬇").right_aligned(), Text::from("Select row")]),
        Row::from_iter([
            Text::from("+/-").right_aligned(),
            Text::from("Change priority"),
        ]),
        Row::from_iter([
            Text::from("1-6").right_aligned(),
            Text::from("Set priority"),
        ]),
        Row::from_iter([
            Text::from("e").right_aligned(),
            Text::from("Edit description"),
        ]),
        Row::from_iter([Text::from("n").right_aligned(), Text::from("Edit note")]),
        Row::from_iter([Text::from("t").right_aligned(), Text::from("Edit tags")]),
        Row::from_iter([Text::from("a").right_aligned(), Text::from("Add new task")]),
        Row::from_iter([Text::from("D").right_aligned(), Text::from("Remove task")]),
        Row::from_iter([Text::from("q").right_aligned(), Text::from("Save and exit")]),
        Row::from_iter([Text::from("X").right_aligned(), Text::from("Exit")]),
    ];
    let ws = [5, 19];
    let width: u16 = ws.iter().sum();
    let height = rows.len() as u16;
    let table = Table::default()
        .block(Block::default().bg(Color::Magenta))
        .widths(ws.map(Constraint::Length))
        .rows(rows);

    let mut size = frame.size();
    let y = size.height.saturating_sub(height);
    size.y = y;
    size.height = height;
    size.width = width;
    frame.render_widget(ratatui::widgets::Clear, size);
    frame.render_widget(table, size)
}
