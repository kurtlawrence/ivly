use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Task<S> {
    id: String,

    pub description: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub note: String,

    /// Seconds since UNIX epoch.
    created: u64,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub tags: Vec<String>,

    state: S,
}

pub type TodoTask = Task<Todo>;
pub type DoneTask = Task<Done>;

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Copy)]
pub struct Todo {
    marked: Option<Done>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
pub struct Done {
    /// Seconds since UNIX epoch.
    completed: u64,
}

impl Done {
    fn duration_since_completed(&self) -> Duration {
        let secs = crate::now().checked_sub(self.completed).unwrap_or_default();
        Duration::from_secs(secs)
    }
}

impl Default for Task<Todo> {
    fn default() -> Self {
        Self {
            id: nanoid::nanoid!(4),
            description: String::new(),
            note: String::new(),
            created: crate::now(),
            tags: Vec::new(),
            state: Todo::default(),
        }
    }
}

impl<S> Task<S> {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn tags(&self) -> impl ExactSizeIterator<Item = &str> {
        self.tags.iter().map(String::as_str)
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    pub fn duration_since_creation(&self) -> Duration {
        let secs = (crate::now() - self.created).max(0);
        Duration::from_secs(secs)
    }
}

impl TodoTask {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            ..Default::default()
        }
    }

    pub fn finish(&mut self) {
        if self.state.marked.is_none() {
            self.state = Todo {
                marked: Some(Done {
                    completed: crate::now(),
                }),
            };
        }
    }

    pub fn is_finished(&self) -> bool {
        self.state.marked.is_some()
    }

    pub fn duration_since_finished(&self) -> Option<Duration> {
        self.state
            .marked
            .as_ref()
            .map(Done::duration_since_completed)
    }

    pub fn complete(self) -> DoneTask {
        let Self {
            id,
            description,
            note,
            created,
            tags,
            state,
        } = self;
        let state = state.marked.unwrap_or_else(|| Done {
            completed: crate::now(),
        });
        DoneTask {
            id,
            description,
            note,
            created,
            tags,
            state,
        }
    }
}

impl DoneTask {
    pub fn duration_since_completed(&self) -> Duration {
        self.state.duration_since_completed()
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Tasks<T>(pub Vec<Task<T>>);

pub type TodoTasks = Tasks<Todo>;
pub type DoneTasks = Tasks<Done>;

impl<T> Deref for Tasks<T> {
    type Target = Vec<Task<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Tasks<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Tasks<T> {
    pub fn new() -> Self {
        Tasks(Vec::new())
    }

    pub fn into_iter(self) -> impl ExactSizeIterator<Item = Task<T>> {
        self.0.into_iter()
    }
}

impl DoneTasks {
    /// Sorts the tasks as most recently closed to oldest closed.
    pub fn sort(&mut self) {
        self.0
            .sort_by(|a, b| b.state.completed.cmp(&a.state.completed))
    }
}
