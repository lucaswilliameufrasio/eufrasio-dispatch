use rusqlite::{Connection, Result, params};

pub(crate) fn ensure_public_ids(path: &str) -> Result<()> {
    let connection = Connection::open(path)?;
    ensure_table_public_ids(&connection, "incidents")?;
    ensure_table_public_ids(&connection, "incident_updates")?;
    Ok(())
}

fn ensure_table_public_ids(connection: &Connection, table: &str) -> Result<()> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info(\"{table}\")"))?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !columns.iter().any(|column| column == "public_id") {
        connection.execute_batch(&format!(
            "ALTER TABLE \"{table}\" ADD COLUMN public_id TEXT NOT NULL DEFAULT ''"
        ))?;
    }

    let mut statement = connection.prepare(&format!(
        "SELECT id FROM \"{table}\" WHERE public_id IS NULL OR public_id = ''"
    ))?;
    let ids = statement
        .query_map([], |row| row.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;

    for id in ids {
        connection.execute(
            &format!("UPDATE \"{table}\" SET public_id = ?1 WHERE id = ?2"),
            params![uuid::Uuid::now_v7().to_string(), id],
        )?;
    }

    connection.execute_batch(&format!(
        "CREATE UNIQUE INDEX IF NOT EXISTS \"idx_{table}_public_id\" ON \"{table}\" (public_id)"
    ))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::ensure_table_public_ids;

    #[test]
    fn backfills_uuid_v7_without_changing_existing_row_ids() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE notes (id INTEGER PRIMARY KEY, title TEXT NOT NULL); INSERT INTO notes (id, title) VALUES (5, 'existing note');",
            )
            .unwrap();

        ensure_table_public_ids(&connection, "notes").unwrap();

        let (id, public_id): (i64, String) = connection
            .query_row("SELECT id, public_id FROM notes", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();
        assert_eq!(id, 5);
        assert_eq!(public_id.chars().nth(14), Some('7'));

        ensure_table_public_ids(&connection, "notes").unwrap();
        let saved: String = connection
            .query_row("SELECT public_id FROM notes WHERE id = 5", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(saved, public_id);
    }
}
