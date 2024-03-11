use crate::{days_ago, tags::Tags, task::TodoTask};
use colored::*;

pub fn todo_task(index: usize, task: &TodoTask, tags: &Tags) {
    let done = task.is_finished();
    print!(
        " {:>4} {}",
        format!("{}.", index + 1).truecolor(127, 127, 127).bold(),
        if done {
            task.description.bold().strikethrough()
        } else {
            task.description.bold()
        }
    );

    if let Some(finished) = task.duration_since_finished() {
        print!(
            " ➡ {}",
            format!("Completed {}", days_ago(finished))
                .green()
                .underline()
        )
    }
    println!();

    if !task.note.is_empty() {
        println!("       {}", task.note.italic());
    }

    print!(
        "       {} ",
        days_ago(task.duration_since_creation())
            .truecolor(165, 165, 165)
            .underline()
    );

    for tag in task.tags() {
        print!("{} ", tags.colourise(tag, tag));
    }

    println!();
}

pub fn tags(tags: &Tags) {
    for (tag, style) in tags.iter() {
        println!(
            "{}\t{}\t{}",
            tags.colourise(tag, tag),
            style.fg,
            style.bg.as_deref().unwrap_or("")
        );
    }
}
