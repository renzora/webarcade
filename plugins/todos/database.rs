use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: i64,
    pub channel: String,
    pub username: String,
    pub task: String,
    pub completed: bool,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub rewarded: bool,
}

pub fn create_todo(
    conn: &Connection,
    channel: &str,
    username: &str,
    task: &str,
) -> Result<i64> {
    let now = current_timestamp();

    conn.execute(
        "INSERT INTO todos (channel, username, task, completed, created_at)
         VALUES (?1, ?2, ?3, 0, ?4)",
        params![channel, username, task, now],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn get_todos(conn: &Connection, channel: &str, completed: Option<bool>) -> Result<Vec<Todo>> {
    let query = match completed {
        Some(true) => "SELECT id, channel, username, task, completed, created_at, completed_at, rewarded
                       FROM todos WHERE channel = ?1 AND completed = 1 ORDER BY completed_at DESC",
        Some(false) => "SELECT id, channel, username, task, completed, created_at, completed_at, rewarded
                        FROM todos WHERE channel = ?1 AND completed = 0 ORDER BY created_at DESC",
        None => "SELECT id, channel, username, task, completed, created_at, completed_at, rewarded
                 FROM todos WHERE channel = ?1 ORDER BY created_at DESC",
    };

    let mut stmt = conn.prepare(query)?;

    let todos = stmt.query_map(params![channel], |row| {
        Ok(Todo {
            id: row.get(0)?,
            channel: row.get(1)?,
            username: row.get(2)?,
            task: row.get(3)?,
            completed: row.get::<_, i64>(4)? != 0,
            created_at: row.get(5)?,
            completed_at: row.get(6)?,
            rewarded: row.get::<_, i64>(7)? != 0,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(todos)
}

pub fn toggle_todo(conn: &Connection, todo_id: i64) -> Result<()> {
    // Get current status
    let completed: i64 = conn.query_row(
        "SELECT completed FROM todos WHERE id = ?1",
        params![todo_id],
        |row| row.get(0),
    )?;

    let new_status = if completed == 0 { 1 } else { 0 };
    let now = current_timestamp();

    if new_status == 1 {
        // Mark as completed
        conn.execute(
            "UPDATE todos SET completed = 1, completed_at = ?1 WHERE id = ?2",
            params![now, todo_id],
        )?;
    } else {
        // Mark as incomplete
        conn.execute(
            "UPDATE todos SET completed = 0, completed_at = NULL WHERE id = ?1",
            params![todo_id],
        )?;
    }

    Ok(())
}

pub fn delete_todo(conn: &Connection, todo_id: i64) -> Result<()> {
    conn.execute("DELETE FROM todos WHERE id = ?1", params![todo_id])?;
    Ok(())
}

pub fn delete_completed_todos(conn: &Connection, channel: &str) -> Result<usize> {
    let count = conn.execute(
        "DELETE FROM todos WHERE channel = ?1 AND completed = 1",
        params![channel],
    )?;
    Ok(count)
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
