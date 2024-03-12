# ivly

Command line tool for tasks following the [Ivy Lee method](https://jamesclear.com/ivy-lee).

The Ivy Lee method is simple.

1. At the end of each work day, write down the six most important things you need to accomplish tomorrow. Do not write down more than six tasks.
2. Prioritize those six items in order of their true importance.
3. When you arrive tomorrow, concentrate only on the first task. Work until the first task is finished before moving on to the second task.
4. Approach the rest of your list in the same fashion. At the end of the day, move any unfinished items to a new list of six tasks for the following day.
5. Repeat this process every working day.

![](use.mp4)

## Install

Install using Rust and Cargo (https://www.rust-lang.org/tools/install).

```sh
cargo install --git https://github.com/kurtlawrence/ivly
```

## Use

### `ivly`

Show the **6** priority tasks.

```sh
ivly
# Filter list with tags
ivly +code /tests
```

### `add`

Add a new task.

```sh
ivly add # add task interactively
ivly add "A task description"
ivly add "A task description" -n "Some note" +tag1 +tag2
```

### `finish`

Finish a task.

```sh
ivly finish 1 # finish the first task
```

### `sweep`

Move all finished tasks into the done list.

```sh
ivly sweep
```

### `bump`

Bump a task to the end of the task list.

```sh
ivly bump 3 # Bumps the 3rd task to the end
```

### `move`

Reprioritise a task.

```sh
ivly move 3 1 # Moves the 3rd task in front of the 1st task
ivly move # enter interactive move mode
```

### `list`

List **all** the tasks in a table.

```sh
ivly list
ivly list --open # list just open tasks
ivly list +foo /bar # list tasks with tag 'foo' but not 'bar'
```

### `tag`

Edit a tag's styling.
See colour names at https://docs.rs/colored/2.1.0/src/colored/color.rs.html#88-111

```sh
ivly tag foo --fg blue --bg red
```

### `edit`

Edit a task.

```sh
ivly edit qw8y -d "new description" -n "new note" +new-tag /remove-tag
```

### `remove`

Remove a task.
This completely deletes the task.

```sh
ivly remove qw8y
```

## Configuration

By default, the tasks are saved in `$HOME/.ivly` in [RON](https://github.com/ron-rs/ron) format.
The save directory can be altered by setting the environment variable `IVLY_DIR`.

For example, I save my tasks to:
```sh
export IVLY_DIR=/stuff/Dropbox/Notes/ivly-tasks
```
