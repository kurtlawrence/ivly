use crate::{
    tags::Tags,
    task::{DoneTasks, Tasks, TodoTasks},
};
use miette::*;
use std::path::Path;

pub fn read_open_tasks(dir: &Path) -> TodoTasks {
    let file = dir.join("open.ron");
    let mut tasks = std::fs::read_to_string(file)
        .ok()
        .and_then(|x| ron::from_str(&x).ok());
    if tasks.is_none() {
        eprintln!("⚠️ Failed to read the saved tasks, opening backup tasks");
        let file = dir.join("open.bak.ron");
        tasks = std::fs::read_to_string(file)
            .ok()
            .and_then(|x| ron::from_str(&x).ok());
    }

    tasks.unwrap_or_else(|| {
        eprintln!("⚠️ No tasks saved, creating a new set");
        Tasks::new()
    })
}

pub fn write_open_tasks(dir: &Path, tasks: &TodoTasks) -> Result<()> {
    let file_bak = dir.join("open.bak.ron");
    let file = dir.join("open.ron");
    let _ = std::fs::copy(&file, file_bak);
    let s = ron::ser::to_string_pretty(tasks, Default::default())
        .into_diagnostic()
        .wrap_err("failed to serialise open tasks")?;
    std::fs::write(file, s.as_bytes()).into_diagnostic()
}

pub fn read_done_tasks(dir: &Path) -> DoneTasks {
    let file = dir.join("done.ron");
    let mut tasks = std::fs::read_to_string(file)
        .ok()
        .and_then(|x| ron::from_str(&x).ok());
    if tasks.is_none() {
        eprintln!("⚠️ Failed to read the saved tasks, opening backup tasks");
        let file = dir.join("done.bak.ron");
        tasks = std::fs::read_to_string(file)
            .ok()
            .and_then(|x| ron::from_str(&x).ok());
    }

    tasks.unwrap_or_else(|| {
        eprintln!("⚠️ No tasks saved, creating a new set");
        Tasks::new()
    })
}

pub fn write_done_tasks(dir: &Path, tasks: &DoneTasks) -> Result<()> {
    let file_bak = dir.join("done.bak.ron");
    let file = dir.join("done.ron");
    let _ = std::fs::copy(&file, file_bak);
    let s = ron::ser::to_string_pretty(tasks, Default::default())
        .into_diagnostic()
        .wrap_err("failed to serialise done tasks")?;
    std::fs::write(file, s.as_bytes()).into_diagnostic()
}

pub fn read_tags(dir: &Path) -> Tags {
    let file = dir.join("tags.ron");
    std::fs::read_to_string(file)
        .ok()
        .and_then(|x| ron::from_str(&x).ok())
        .unwrap_or_default()
}

pub fn write_tags(dir: &Path, tags: &Tags) -> Result<()> {
    let file = dir.join("tags.ron");
    let s = ron::ser::to_string_pretty(tags, Default::default())
        .into_diagnostic()
        .wrap_err("failed to serialise tags")?;
    std::fs::write(file, s.as_bytes()).into_diagnostic()
}

pub fn read_last_tags(dir: &Path) -> Vec<String> {
    let file = dir.join("last-tags.ron");
    std::fs::read_to_string(file)
        .ok()
        .and_then(|x| ron::from_str(&x).ok())
        .unwrap_or_default()
}

pub fn write_last_tags(dir: &Path, tags: &[String]) -> Result<()> {
    let file = dir.join("last-tags.ron");
    let s = ron::ser::to_string_pretty(tags, Default::default())
        .into_diagnostic()
        .wrap_err("failed to serialise tags")?;
    std::fs::write(file, s.as_bytes()).into_diagnostic()
}
