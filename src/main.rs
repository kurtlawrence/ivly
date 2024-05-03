#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod io;
mod op;
mod print;
mod tags;
mod task;
#[cfg(test)]
mod tests;
mod tui;

use clap::{Parser, Subcommand};
use colored::*;
use miette::IntoDiagnostic;
use std::time::Duration;
use tags::{AddTag, FilterTag};

fn main() -> miette::Result<()> {
    let app = App::parse();

    let dir = &if cfg!(debug_assertions) {
        "./target/.ivly".to_string()
    } else {
        std::env::var("IVLY_DIR").unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|x| format!("{x}/.ivly"))
                .unwrap_or_else(|_| String::from(".ivly"))
        })
    };
    std::fs::create_dir_all(dir).into_diagnostic()?;

    let dir: &std::path::Path = dir.as_ref();

    match app.cmd {
        None => {
            let tasks = io::read_open_tasks(dir);
            let tags = io::read_tags(dir);
            let mut ts = tasks
                .iter()
                .enumerate()
                .filter(|(_, task)| app.tags.iter().all(|f| f.filter(task.tags())));

            ts.by_ref()
                .take(6)
                .for_each(|(i, t)| print::todo_task(i, t, &tags));
            let rem = ts.count();
            if rem > 0 {
                println!();
                println!(
                    "      {}",
                    format!("{rem} tasks in backlog")
                        .italic()
                        .truecolor(127, 127, 127)
                );
            }
        }
        Some(Cmd::Add {
            description,
            note,
            tags,
            tui,
        }) => {
            if tui {
                op::move_interactive(dir)
            } else {
                match description {
                    Some(desc) => op::add(dir, desc, note, tags),
                    None => op::add_interactive(dir),
                }
            }
        }?,
        Some(Cmd::Finish { task_num }) => {
            if task_num.is_empty() {
                op::finish(dir, None)?;
            } else {
                for n in task_num {
                    op::finish(dir, n.into())?;
                }
            }
        }
        Some(Cmd::Sweep) => op::sweep(dir)?,
        Some(Cmd::Bump { mut task_num }) => {
            task_num.sort_unstable();
            task_num.dedup();
            for task_num in task_num.into_iter().rev() {
                op::bump(dir, task_num)?;
            }
        }
        Some(Cmd::Move {
            task_num,
            insert_before,
        }) => match (task_num, insert_before) {
            (Some(task_num), Some(insert_before)) => op::move_(dir, task_num, insert_before),
            (None, None) => op::move_interactive(dir),
            _ => Err(miette::miette!(
                "please specify both a task number and the number to insert before"
            )),
        }?,
        Some(Cmd::List { open, done, tags }) => op::list(dir, open, done, tags),
        Some(Cmd::Tag { tag, fg, bg }) => op::edit_tag(dir, &tag, fg, bg)?,
        Some(Cmd::Edit { task_id: None, .. }) => op::move_interactive(&dir)?,
        Some(Cmd::Edit {
            task_id: Some(task_id),
            desc,
            note,
            tags,
        }) => op::edit(dir, &task_id, desc, note, tags)?,
        Some(Cmd::Remove { task_id }) => op::remove(dir, &task_id)?,
    }

    Ok(())
}

/// Main ivly CLI app.
#[derive(Parser)]
#[clap(version, author)]
#[clap(about = "\
Command line tasks following the Ivy Lee method.
https://github.com/kurtlawrence/ivly
")]
#[clap(help_template = "\
{before-help}{name} {version}
{about}
{usage-heading} {usage}

{all-args}{after-help}")]
pub struct App {
    /// Optional subcommand.
    #[clap(subcommand)]
    pub cmd: Option<Cmd>,
    /// When used with `ivly`, apply filter tags to reduce todo task list.
    /// + to include tag.
    /// / to exclude tag.
    tags: Vec<FilterTag>,
}

/// Subcommand for operations.
#[derive(Subcommand)]
pub enum Cmd {
    /// Add a new task.
    /// If no description specified, enters interactive add mode.
    #[command(alias("a"))]
    Add {
        /// The task description.
        description: Option<String>,
        /// The task note.
        #[clap(short, long)]
        note: Option<String>,
        /// Task tags.
        /// Tags should be prefixed with +.
        tags: Vec<AddTag>,
        /// Use an interactive adding TUI.
        #[arg(long, short('i'))]
        tui: bool,
    },

    /// Finish a task.
    #[command(alias("f"))]
    Finish {
        /// The task number. If not specified, finishes the **first** available task.
        task_num: Vec<usize>,
    },

    /// Move finished tasks into done list.
    Sweep,

    /// Bump a task to the end of the open list.
    Bump {
        /// The task number.
        task_num: Vec<usize>,
    },

    /// Move a task.
    /// If no task numbers are specified, enters interactive move mode.
    #[command(alias("mv"))]
    Move {
        /// The task number.
        task_num: Option<usize>,
        /// The task to insert *before*.
        insert_before: Option<usize>,
    },

    /// List the tasks.
    #[command(alias("ls"))]
    List {
        /// Only show open tasks.
        #[clap(long)]
        open: bool,
        /// Only show done tasks.
        #[clap(long)]
        done: bool,
        /// Filter by tags.
        /// + to include tag.
        /// / to exclude tag.
        tags: Vec<FilterTag>,
    },

    /// Set the styling of a tag.
    /// See colour names at https://docs.rs/colored/2.1.0/src/colored/color.rs.html#88-111
    Tag {
        /// The tag.
        tag: String,
        /// The foreground colour.
        #[clap(long)]
        fg: Option<colored::Color>,
        /// The background colour.
        #[clap(long)]
        bg: Option<colored::Color>,
    },

    /// Edit a task's description, note, and/or tags.
    /// If no id is given, drops into the TUI editor.
    Edit {
        /// The task ID.
        task_id: Option<String>,
        /// Set the tasks description.
        #[clap(short, long)]
        desc: Option<String>,
        /// Set the tasks note.
        #[clap(short, long)]
        note: Option<String>,
        /// Add or remove tags.
        tags: Vec<FilterTag>,
    },

    /// Remove a task, deleting it completely.
    Remove {
        /// The task ID to remove.
        task_id: String,
    },
}

/// Seconds since the UNIX epoch
fn now() -> u64 {
    use std::time::*;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn days_ago(duration: Duration) -> String {
    let x = humantime::format_duration(duration).to_string();
    let x = x.split(' ').next().unwrap_or_default();
    format!("{x} ago")
}

fn tag_csv<'a>(tags: impl Iterator<Item = &'a str>) -> String {
    let mut x = tags.fold(String::new(), |s, x| s + x + ",");
    x.pop();
    x
}
