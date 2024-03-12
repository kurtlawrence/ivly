use crate::{
    days_ago, io, print, tag_csv,
    tags::{AddTag, FilterTag, Tags},
    task::{TodoTask, TodoTasks},
    tui,
};
use miette::*;
use std::{io::Write, ops::Not, path::Path};

fn ask(question: &str) -> Result<String> {
    let stdout = &mut std::io::stdout();
    write!(stdout, "{question} ").into_diagnostic()?;
    stdout.flush().into_diagnostic()?;
    let mut resp = String::new();
    std::io::stdin().read_line(&mut resp).into_diagnostic()?;
    let resp = resp.trim().to_string();
    Ok(resp)
}

pub fn add(dir: &Path, description: String, note: Option<String>, tags: Vec<AddTag>) -> Result<()> {
    let mut task = TodoTask::new(description);
    if let Some(note) = note {
        task.note = note;
    }
    for tag in tags {
        task.add_tag(tag);
    }
    let mut tasks = io::read_open_tasks(dir);
    let tags = io::read_tags(dir);
    tasks.push(task);

    io::write_open_tasks(dir, &tasks)?;

    let (i, task) = tasks.iter().enumerate().last().unwrap();
    println!("✅ Added new task! ID: {}", task.id());
    print::todo_task(i, task, &tags);
    Ok(())
}

pub fn add_interactive(dir: &Path) -> Result<()> {
    let desc = ask("Task description:")?;
    let note = ask("Task note:")?;
    let tags = ask("Tags:")?;
    let mut ts = Vec::new();
    for tag in tags.split(' ') {
        let tag = tag.parse().map_err(|e| miette!("{e}"))?;
        ts.push(tag);
    }
    add(dir, desc, note.is_empty().not().then_some(note), ts)
}

fn read_tasks_tags(dir: &Path) -> (TodoTasks, Tags) {
    (io::read_open_tasks(dir), io::read_tags(dir))
}

fn translate_task_num(tasks: &TodoTasks, num: usize) -> Result<usize> {
    let r = 1..=tasks.len();
    ensure!(
        r.contains(&num),
        "task number {num} is not within task range {r:#?}"
    );
    Ok(num - 1)
}

pub fn finish(dir: &Path, task_num: Option<usize>) -> Result<()> {
    let (mut tasks, tags) = read_tasks_tags(dir);
    let task_num = task_num.unwrap_or_else(|| {
        tasks
            .iter()
            .position(|t| !t.is_finished())
            .unwrap_or_default()
            + 1
    });
    let index = translate_task_num(&tasks, task_num)?;
    let task = tasks.get_mut(index).unwrap();
    task.finish();
    let task = task.clone();
    io::write_open_tasks(dir, &tasks)?;
    println!("✅ Finished '{}'!", task.description);
    tasks
        .iter()
        .enumerate()
        .take(6)
        .for_each(|(i, t)| print::todo_task(i, t, &tags));
    Ok(())
}

pub fn sweep(dir: &Path) -> Result<()> {
    let (mut open, tags) = read_tasks_tags(dir);
    let mut done = io::read_done_tasks(dir);

    let mut i = 0;
    while i < open.len() {
        if open[i].is_finished() {
            let val = open.remove(i);
            done.push(val.complete());
        } else {
            i += 1;
        }
    }

    done.sort();

    io::write_done_tasks(dir, &done)?;
    io::write_open_tasks(dir, &open)?;

    println!("✅ Swept finished tasks into done list");
    open.iter()
        .enumerate()
        .take(6)
        .for_each(|(i, t)| print::todo_task(i, t, &tags));
    Ok(())
}

pub fn bump(dir: &Path, task_num: usize) -> Result<()> {
    let (mut tasks, tags) = read_tasks_tags(dir);
    let index = translate_task_num(&tasks, task_num)?;
    let task = tasks.remove(index);
    tasks.push(task);
    io::write_open_tasks(dir, &tasks)?;
    let task = tasks.last().unwrap();
    println!("✅ Bumped '{}'!", task.description);
    tasks
        .iter()
        .enumerate()
        .last()
        .into_iter()
        .for_each(|(i, t)| print::todo_task(i, t, &tags));
    Ok(())
}

pub fn move_(dir: &Path, task_num: usize, insert_before: usize) -> Result<()> {
    let mut tasks = io::read_open_tasks(dir);
    let task = translate_task_num(&tasks, task_num)?;
    let mut before = translate_task_num(&tasks, insert_before)?;
    if task < before {
        before = before.saturating_sub(1);
    }
    let task = tasks.remove(task);
    tasks.insert(before, task);
    io::write_open_tasks(dir, &tasks)?;
    let (a, b) = (&tasks[before], &tasks[before + 1]);
    println!(
        "✅ Moved '{}' in front of '{}'!",
        a.description, b.description
    );
    Ok(())
}

pub fn move_interactive(dir: &Path) -> Result<()> {
    let mut tasks = io::read_open_tasks(dir);
    let save = tui::Move::new(&mut tasks).run()?;

    if save {
        io::write_open_tasks(dir, &tasks)?;
        println!("✅ Saved changes");
    } else {
        println!("No changes made");
    }
    Ok(())
}

pub fn list(dir: &Path, only_open: bool, only_done: bool, tags: Vec<FilterTag>) {
    let fopen = only_open || !(only_open ^ only_done);
    let fdone = only_done || !(only_open ^ only_done);

    let open = io::read_open_tasks(dir)
        .into_iter()
        .filter(|_| fopen)
        .filter(|t| tags.iter().all(|f| f.filter(t.tags())));
    let done = io::read_done_tasks(dir)
        .into_iter()
        .filter(|_| fdone)
        .filter(|t| tags.iter().all(|f| f.filter(t.tags())));

    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_HORIZONTAL_ONLY)
        .set_header([
            "ID",
            "Task#",
            "Description",
            "Note",
            "Status",
            "Created",
            "Finished",
            "Tags",
        ]);

    table.add_rows(open.enumerate().map(|(i, t)| {
        [
            t.id().to_string(),
            format!("{}", i + 1),
            t.description.clone(),
            t.note.clone(),
            if t.is_finished() {
                "marked".to_string()
            } else {
                "todo".to_string()
            },
            days_ago(t.duration_since_creation()),
            t.duration_since_finished()
                .map(days_ago)
                .unwrap_or_default(),
            tag_csv(t.tags()),
        ]
    }));

    table.add_rows(done.map(|t| {
        [
            t.id().to_string(),
            String::new(),
            t.description.clone(),
            t.note.clone(),
            "done".to_string(),
            days_ago(t.duration_since_creation()),
            days_ago(t.duration_since_completed()),
            tag_csv(t.tags()),
        ]
    }));

    println!("{table}");
}

pub fn edit_tag(
    dir: &Path,
    tag: &str,
    fg: Option<colored::Color>,
    bg: Option<colored::Color>,
) -> Result<()> {
    let mut tags = io::read_tags(dir);
    if let Some(fg) = fg {
        tags.set_fg(tag, fg);
    }
    if let Some(bg) = bg {
        tags.set_bg(tag, bg);
    }

    io::write_tags(dir, &tags)?;
    print::tags(&tags);
    Ok(())
}

pub fn edit(
    dir: &Path,
    id: &str,
    description: Option<String>,
    note: Option<String>,
    tags: Vec<FilterTag>,
) -> Result<()> {
    let mut tasks = io::read_open_tasks(dir);
    let task = tasks.iter_mut().find(|t| t.id() == id);
    if let Some(task) = task {
        if let Some(d) = description {
            task.description = d;
        }
        if let Some(n) = note {
            task.note = n;
        }
        for t in tags {
            if t.is_neg() {
                task.remove_tag(&t);
            } else {
                task.add_tag(t);
            }
        }
        io::write_open_tasks(dir, &tasks)?;
        println!("✅ Edited task {id}");
        return Ok(());
    }

    let mut tasks = io::read_done_tasks(dir);
    let task = tasks.iter_mut().find(|t| t.id() == id);
    if let Some(task) = task {
        if let Some(d) = description {
            task.description = d;
        }
        if let Some(n) = note {
            task.note = n;
        }
        for t in tags {
            task.add_tag(t);
        }
        io::write_done_tasks(dir, &tasks)?;
        println!("✅ Edited task {id}");
        return Ok(());
    }

    Err(miette!("No task found with ID '{id}'"))
}

pub fn remove(dir: &Path, id: &str) -> Result<()> {
    let mut tasks = io::read_open_tasks(dir);
    let ol1 = tasks.len();
    tasks.retain(|t| t.id() != id);
    let nl1 = tasks.len();
    io::write_open_tasks(dir, &tasks)?;

    let mut tasks = io::read_done_tasks(dir);
    let ol2 = tasks.len();
    tasks.retain(|t| t.id() != id);
    let nl2 = tasks.len();
    io::write_done_tasks(dir, &tasks)?;

    if ol1 != nl1 {
        println!("✅ Removed task `{id}` from todo task list");
    } else if ol2 != nl2 {
        println!("✅ Removed task `{id}` from done task list");
    } else {
        return Err(miette!("task `{id}` not found in todo or done task lists"));
    }
    Ok(())
}
