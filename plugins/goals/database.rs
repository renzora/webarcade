use rusqlite::{Connection, Result, params, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: i64,
    pub channel: String,
    pub title: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub goal_type: String,
    pub target: i64,
    pub current: i64,
    pub is_sub_goal: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn create_goal(
    conn: &Connection,
    channel: &str,
    title: &str,
    description: Option<&str>,
    goal_type: &str,
    target: i64,
    is_sub_goal: bool,
) -> Result<i64> {
    let now = current_timestamp();

    conn.execute(
        "INSERT INTO goals (channel, title, description, type, target, current, is_sub_goal, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?7)",
        params![channel, title, description, goal_type, target, is_sub_goal as i64, now],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn get_goals(conn: &Connection, channel: &str) -> Result<Vec<Goal>> {
    let mut stmt = conn.prepare(
        "SELECT id, channel, title, description, type, target, current, is_sub_goal, created_at, updated_at
         FROM goals WHERE channel = ?1 ORDER BY created_at DESC"
    )?;

    let goals = stmt.query_map(params![channel], |row| {
        Ok(Goal {
            id: row.get(0)?,
            channel: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            goal_type: row.get(4)?,
            target: row.get(5)?,
            current: row.get(6)?,
            is_sub_goal: row.get::<_, i64>(7)? != 0,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(goals)
}

pub fn get_goal(conn: &Connection, goal_id: i64) -> Result<Option<Goal>> {
    conn.query_row(
        "SELECT id, channel, title, description, type, target, current, is_sub_goal, created_at, updated_at
         FROM goals WHERE id = ?1",
        params![goal_id],
        |row| Ok(Goal {
            id: row.get(0)?,
            channel: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            goal_type: row.get(4)?,
            target: row.get(5)?,
            current: row.get(6)?,
            is_sub_goal: row.get::<_, i64>(7)? != 0,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        }),
    ).optional()
}

pub fn update_progress(conn: &Connection, goal_id: i64, amount: i64) -> Result<()> {
    let now = current_timestamp();

    conn.execute(
        "UPDATE goals SET current = current + ?1, updated_at = ?2 WHERE id = ?3",
        params![amount, now, goal_id],
    )?;

    Ok(())
}

pub fn set_progress(conn: &Connection, goal_id: i64, current: i64) -> Result<()> {
    let now = current_timestamp();

    conn.execute(
        "UPDATE goals SET current = ?1, updated_at = ?2 WHERE id = ?3",
        params![current, now, goal_id],
    )?;

    Ok(())
}

pub fn update_goal(
    conn: &Connection,
    goal_id: i64,
    title: Option<&str>,
    description: Option<&str>,
    goal_type: Option<&str>,
    target: Option<i64>,
    current: Option<i64>,
    is_sub_goal: Option<bool>,
) -> Result<()> {
    let now = current_timestamp();

    // Build dynamic UPDATE query
    let mut updates = vec!["updated_at = ?1".to_string()];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now)];

    if let Some(t) = title {
        updates.push(format!("title = ?{}", params.len() + 1));
        params.push(Box::new(t.to_string()));
    }
    if let Some(d) = description {
        updates.push(format!("description = ?{}", params.len() + 1));
        params.push(Box::new(d.to_string()));
    }
    if let Some(gt) = goal_type {
        updates.push(format!("type = ?{}", params.len() + 1));
        params.push(Box::new(gt.to_string()));
    }
    if let Some(tgt) = target {
        updates.push(format!("target = ?{}", params.len() + 1));
        params.push(Box::new(tgt));
    }
    if let Some(cur) = current {
        updates.push(format!("current = ?{}", params.len() + 1));
        params.push(Box::new(cur));
    }
    if let Some(is_sub) = is_sub_goal {
        updates.push(format!("is_sub_goal = ?{}", params.len() + 1));
        params.push(Box::new(is_sub as i64));
    }

    let query = format!(
        "UPDATE goals SET {} WHERE id = ?{}",
        updates.join(", "),
        params.len() + 1
    );
    params.push(Box::new(goal_id));

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    conn.execute(&query, params_refs.as_slice())?;

    Ok(())
}

pub fn delete_goal(conn: &Connection, goal_id: i64) -> Result<()> {
    conn.execute("DELETE FROM goals WHERE id = ?1", params![goal_id])?;
    Ok(())
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
