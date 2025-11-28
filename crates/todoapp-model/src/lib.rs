use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: Priority,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Todo {
    pub fn new(
        title: String,
        description: Option<String>,
        due_date: Option<DateTime<Utc>>,
        priority: Priority,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            due_date,
            priority,
            completed: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn mark_completed(&mut self) {
        self.completed = true;
        self.updated_at = Utc::now();
    }

    pub fn mark_incomplete(&mut self) {
        self.completed = false;
        self.updated_at = Utc::now();
    }

    pub fn update(
        &mut self,
        title: Option<String>,
        description: Option<Option<String>>,
        due_date: Option<Option<DateTime<Utc>>>,
        priority: Option<Priority>,
    ) {
        if let Some(t) = title {
            self.title = t;
        }
        if let Some(d) = description {
            self.description = d;
        }
        if let Some(dd) = due_date {
            self.due_date = dd;
        }
        if let Some(p) = priority {
            self.priority = p;
        }
        self.updated_at = Utc::now();
    }
}

mod db;
pub use db::TodoDb;
