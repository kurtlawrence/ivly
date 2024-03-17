use crate::{io, op, print, tags::Tags};
use std::path::Path;

#[test]
fn main_integration_test() {
    colored::control::set_override(true); // always colour for testing

    let dir = Path::new("./target/integration-test");
    std::fs::remove_dir_all(dir).ok();
    std::fs::create_dir_all(dir).unwrap();

    let mut settings = insta::Settings::clone_current();
    settings.add_redaction("[].id", "[id]");
    settings.add_redaction("[].created", "[created]");
    settings.add_redaction("[].state.completed", "[completed]");
    settings.add_redaction("[].state.marked.completed", "[completed]");
    let _settings = settings.bind_to_scope();

    op::add(dir, "This is a new task".into(), None, Vec::new()).unwrap();
    let tasks = io::read_open_tasks(dir);
    insta::assert_ron_snapshot!(tasks);

    op::add(
        dir,
        "This is a new task 2".into(),
        Some("with a note".to_string()),
        vec!["tag1".into(), "tag-2".into()],
    )
    .unwrap();
    let tasks = io::read_open_tasks(dir);
    insta::assert_ron_snapshot!(tasks);

    op::finish(dir, None).unwrap();
    let open = io::read_open_tasks(dir);
    let done = io::read_done_tasks(dir);
    insta::assert_ron_snapshot!(open);
    insta::assert_ron_snapshot!(done);

    op::sweep(dir).unwrap();
    let open = io::read_open_tasks(dir);
    let done = io::read_done_tasks(dir);
    insta::assert_ron_snapshot!(open);
    insta::assert_ron_snapshot!(done);

    op::add(dir, "This is a new task 3".into(), None, Vec::new()).unwrap();
    op::bump(dir, 1).unwrap();
    let open = io::read_open_tasks(dir);
    insta::assert_ron_snapshot!(open);

    op::move_(dir, 2, 1).unwrap();
    let open = io::read_open_tasks(dir);
    insta::assert_ron_snapshot!(open);

    op::finish(dir, Some(2)).unwrap();
    let open = io::read_open_tasks(dir);
    insta::assert_ron_snapshot!(open);

    op::edit_tag(
        dir,
        "tag-2",
        Some(colored::Color::Green),
        Some(colored::Color::Red),
    )
    .unwrap();
    let tags = io::read_tags(dir);
    insta::assert_ron_snapshot!(tags);
}

#[test]
fn cli_tests() {
    let cmd = || assert_cmd::Command::cargo_bin("ivly").unwrap();
    cmd().arg("add").arg("Hello, world!").assert().success();
    cmd().arg("a").arg("Hello, world!").assert().success();
    cmd().arg("finish").assert().success();
    cmd().arg("f").assert().success();
    cmd().arg("move").args(["1", "2"]).assert().success();
    cmd().arg("mv").args(["1", "2"]).assert().success();
    cmd().arg("list").assert().success();
    cmd().arg("ls").assert().success();
    cmd().arg("f").args(["1", "2"]).assert().success();
    cmd().arg("bump").args(["1", "2"]).assert().success();
}

#[test]
fn print_tags() {
    let mut tags = Tags::default();
    tags.set_fg("tag1", colored::Color::Blue);
    tags.set_fg("tag2", colored::Color::Red);
    tags.set_bg("tag2", colored::Color::Blue);
    tags.set_bg("tag3", colored::Color::Green);

    let mut o = Vec::new();
    print::tags(&tags, &mut o);
    let o = String::from_utf8(o).unwrap();

    insta::assert_snapshot!(o);
}
