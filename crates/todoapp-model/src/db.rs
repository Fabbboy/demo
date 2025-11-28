use crate::Todo;
use anyhow::{Context, Result};
use sled::Db;
use uuid::Uuid;

pub struct TodoDb {
    db: Db,
}

impl TodoDb {
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let db = sled::open(path).context("Failed to open sled database")?;
        Ok(Self { db })
    }

    pub fn insert(&self, todo: &Todo) -> Result<()> {
        let key = todo.id.as_bytes();
        let config = bincode::config::standard();
        let value =
            bincode::serde::encode_to_vec(todo, config).context("Failed to serialize todo")?;
        self.db
            .insert(key, value)
            .context("Failed to insert todo")?;
        self.db.flush().context("Failed to flush database")?;
        Ok(())
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<Todo>> {
        let key = id.as_bytes();
        match self.db.get(key).context("Failed to get todo")? {
            Some(bytes) => {
                let config = bincode::config::standard();
                let (todo, _): (Todo, _) = bincode::serde::decode_from_slice(&bytes, config)
                    .context("Failed to deserialize todo")?;
                Ok(Some(todo))
            }
            None => Ok(None),
        }
    }

    pub fn get_all(&self) -> Result<Vec<Todo>> {
        let mut todos = Vec::new();
        let config = bincode::config::standard();
        for item in self.db.iter() {
            let (_key, value) = item.context("Failed to iterate over todos")?;
            let (todo, _): (Todo, _) = bincode::serde::decode_from_slice(&value, config)
                .context("Failed to deserialize todo")?;
            todos.push(todo);
        }
        // Sort by created_at descending (newest first)
        todos.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(todos)
    }

    pub fn update(&self, todo: &Todo) -> Result<()> {
        let key = todo.id.as_bytes();
        let config = bincode::config::standard();
        let value =
            bincode::serde::encode_to_vec(todo, config).context("Failed to serialize todo")?;
        self.db
            .insert(key, value)
            .context("Failed to update todo")?;
        self.db.flush().context("Failed to flush database")?;
        Ok(())
    }

    pub fn delete(&self, id: &Uuid) -> Result<bool> {
        let key = id.as_bytes();
        let existed = self
            .db
            .remove(key)
            .context("Failed to delete todo")?
            .is_some();
        self.db.flush().context("Failed to flush database")?;
        Ok(existed)
    }

    pub fn clear_all(&self) -> Result<()> {
        self.db.clear().context("Failed to clear database")?;
        self.db.flush().context("Failed to flush database")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Priority;

    #[test]
    fn test_todo_crud() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db = TodoDb::new(temp_dir.path()).unwrap();

        // Create
        let todo = Todo::new(
            "Test todo".to_string(),
            Some("Description".to_string()),
            None,
            Priority::High,
        );
        let id = todo.id;
        db.insert(&todo).unwrap();

        // Read
        let retrieved = db.get(&id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Test todo");
        assert_eq!(retrieved.priority, Priority::High);

        // Update
        let mut updated_todo = retrieved.clone();
        updated_todo.mark_completed();
        db.update(&updated_todo).unwrap();

        let retrieved_again = db.get(&id).unwrap().unwrap();
        assert!(retrieved_again.completed);

        // Delete
        let deleted = db.delete(&id).unwrap();
        assert!(deleted);
        assert!(db.get(&id).unwrap().is_none());
    }

    #[test]
    fn test_get_all() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db = TodoDb::new(temp_dir.path()).unwrap();

        let todo1 = Todo::new("Todo 1".to_string(), None, None, Priority::Low);
        let todo2 = Todo::new("Todo 2".to_string(), None, None, Priority::Medium);

        db.insert(&todo1).unwrap();
        db.insert(&todo2).unwrap();

        let all_todos = db.get_all().unwrap();
        assert_eq!(all_todos.len(), 2);
    }
}
