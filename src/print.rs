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

pub fn tags(tags: &Tags, mut wtr: impl std::io::Write) {
    let ts = tags
        .iter()
        .map(|(tag, _)| (tag.chars().count(), tags.colourise(tag, tag)))
        .collect::<Vec<_>>();
    let fgs = tags
        .iter()
        .map(|(_, style)| (style.fg.chars().count(), style.fg.as_str()))
        .collect::<Vec<_>>();
    let bgs = tags
        .iter()
        .map(|(_, style)| {
            (
                style
                    .bg
                    .as_ref()
                    .map(|x| x.chars().count())
                    .unwrap_or_default(),
                style.bg.as_deref().unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>();

    let lens = [
        ts.iter().map(|x| x.0).max().unwrap_or_default().max(3),
        fgs.iter().map(|x| x.0).max().unwrap_or_default().max(2),
        bgs.iter().map(|x| x.0).max().unwrap_or_default().max(2),
    ];

    fn pad(mut w: impl std::io::Write, n: usize) {
        w.write_all(&vec![b' '; n]).unwrap();
    }

    write!(&mut wtr, "Tag").unwrap();
    pad(&mut wtr, lens[0] - 3);
    write!(&mut wtr, " FG").unwrap();
    pad(&mut wtr, lens[1] - 2);
    write!(&mut wtr, " BG").unwrap();
    pad(&mut wtr, lens[2] - 2);
    writeln!(&mut wtr).unwrap();

    for (((tl, tag), (fl, fg)), (bl, bg)) in ts.into_iter().zip(fgs).zip(bgs) {
        pad(&mut wtr, lens[0] - tl);
        write!(&mut wtr, "{}", tag).unwrap();
        write!(&mut wtr, " {}", fg.color(fg)).unwrap();
        pad(&mut wtr, lens[1] - fl);
        write!(&mut wtr, " {}", bg.on_color(bg)).unwrap();
        pad(&mut wtr, lens[2] - bl);
        writeln!(&mut wtr).unwrap();
    }
}
