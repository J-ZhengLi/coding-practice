use sqlx::{SqlitePool, Row, sqlite::SqliteRow, AssertSqlSafe};
use anyhow::{Result, Context, bail};
use serde_json::{Value, json, Map};
use chrono::Utc;

/// Tables to export/import, in dependency order (materials before exercises due to FK)
const TABLES: [&str; 5] = [
    "config",
    "materials",
    "exercises",
    "submissions",
    "review_schedule",
];

const EXPORT_VERSION: &str = "1.0";

pub struct DataExportService {
    pool: SqlitePool,
}

impl DataExportService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Export all tables as a JSON value with version and timestamp metadata.
    pub async fn export_all(&self) -> Result<Value> {
        let mut tables_data = Map::new();

        for table in TABLES {
            let rows = self.export_table(table).await
                .with_context(|| format!("Failed to export table: {}", table))?;
            tables_data.insert(table.to_string(), rows);
        }

        Ok(json!({
            "version": EXPORT_VERSION,
            "exported_at": Utc::now().to_rfc3339(),
            "tables": tables_data,
        }))
    }

    /// Import data from a JSON value, replacing all tables atomically.
    /// Validates structure before applying any changes.
    pub async fn import_all(&self, data: &Value) -> Result<()> {
        self.validate_import_data(data)?;

        let tables_obj = data["tables"].as_object()
            .context("tables field is not an object")?;

        // Begin transaction for atomic import
        let mut tx = self.pool.begin().await
            .context("Failed to begin import transaction")?;

        // Disable foreign key checks during import so we can clear tables
        // in any order
        sqlx::query("PRAGMA foreign_keys = OFF")
            .execute(&mut *tx)
            .await
            .context("Failed to disable foreign keys")?;

        // Clear and repopulate each table
        for table in TABLES {
            // Delete all existing rows
            let delete_sql = format!("DELETE FROM {}", table);
            sqlx::query(AssertSqlSafe(delete_sql))
                .execute(&mut *tx)
                .await
                .with_context(|| format!("Failed to clear table: {}", table))?;

            // Insert rows from import data
            if let Some(rows) = tables_obj.get(table).and_then(|v| v.as_array()) {
                for row in rows {
                    self.insert_row(&mut tx, table, row).await
                        .with_context(|| format!("Failed to insert row into table: {}", table))?;
                }
            }
        }

        // Re-enable foreign key checks
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut *tx)
            .await
            .context("Failed to re-enable foreign keys")?;

        // Commit transaction
        tx.commit().await
            .context("Failed to commit import transaction")?;

        Ok(())
    }

    /// Export a single table as a JSON array of row objects.
    async fn export_table(&self, table: &str) -> Result<Value> {
        let columns = self.get_table_columns(table).await?;
        let select_sql = format!("SELECT * FROM {}", table);
        let rows = sqlx::query(AssertSqlSafe(select_sql))
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("Failed to query table: {}", table))?;

        let json_rows: Vec<Value> = rows.iter()
            .map(|row| Self::row_to_json(row, &columns))
            .collect();

        Ok(Value::Array(json_rows))
    }

    /// Get column names for a table via PRAGMA table_info.
    async fn get_table_columns(&self, table: &str) -> Result<Vec<String>> {
        let pragma_sql = format!("PRAGMA table_info({})", table);
        let rows = sqlx::query(AssertSqlSafe(pragma_sql))
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("Failed to get column info for table: {}", table))?;

        Ok(rows.iter()
            .filter_map(|row| row.try_get::<String, _>("name").ok())
            .collect())
    }

    /// Convert a SqliteRow to a JSON object using the provided column names.
    fn row_to_json(row: &SqliteRow, columns: &[String]) -> Value {
        let mut map = Map::new();
        for col in columns {
            let value = Self::column_to_json(row, col);
            map.insert(col.clone(), value);
        }
        Value::Object(map)
    }

    /// Convert a single column value from a row to a JSON value.
    /// Attempts type inference: i64, f64, bool, then falls back to string.
    fn column_to_json(row: &SqliteRow, col: &str) -> Value {
        // Try integer first
        if let Ok(v) = row.try_get::<i64, _>(col) {
            return json!(v);
        }
        // Try float
        if let Ok(v) = row.try_get::<f64, _>(col) {
            return json!(v);
        }
        // Try boolean (SQLite stores as 0/1 integer, but we try anyway)
        if let Ok(v) = row.try_get::<bool, _>(col) {
            return json!(v);
        }
        // Try string (this also catches datetime values stored as text)
        if let Ok(v) = row.try_get::<String, _>(col) {
            return json!(v);
        }
        // Null for missing/unreadable values
        Value::Null
    }

    /// Insert a single JSON row into a table using dynamic column names.
    async fn insert_row(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        table: &str,
        row: &Value,
    ) -> Result<()> {
        let obj = row.as_object()
            .context("Row is not a JSON object")?;

        if obj.is_empty() {
            return Ok(()); // Empty object, nothing to insert
        }

        // Validate column names against the actual table schema to prevent SQL injection
        let valid_columns = self.get_table_columns(table).await?;
        let valid_set: std::collections::HashSet<&str> = valid_columns.iter().map(|s| s.as_str()).collect();

        let columns: Vec<&str> = obj.keys()
            .map(|k| k.as_str())
            .filter(|k| valid_set.contains(k))
            .collect();

        if columns.is_empty() {
            return Ok(()); // No valid columns, nothing to insert
        }
        let placeholders: Vec<&str> = columns.iter().map(|_| "?").collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.join(", "),
            placeholders.join(", "),
        );

        // Convert JSON values to string for binding (SQLite will do type affinity)
        let str_values: Vec<String> = columns.iter()
            .map(|col| {
                let value = &obj[*col];
                match value {
                    Value::Null => String::new(),
                    Value::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
                    Value::Number(n) => n.to_string(),
                    Value::String(s) => s.clone(),
                    Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
                }
            })
            .collect();

        // Build query with dynamic binds
        let mut query = sqlx::query(AssertSqlSafe(sql));
        for val in &str_values {
            query = query.bind(val.clone());
        }
        query.execute(&mut **tx).await?;

        Ok(())
    }

    /// Validate the import JSON structure before applying changes.
    fn validate_import_data(&self, data: &Value) -> Result<()> {
        // Check top-level is an object
        let obj = data.as_object()
            .context("Import data must be a JSON object")?;

        // Check version field
        let version = obj.get("version")
            .and_then(|v| v.as_str())
            .context("Missing or invalid 'version' field")?;

        if version != EXPORT_VERSION {
            bail!(
                "Unsupported export version: '{}'. Expected '{}'",
                version,
                EXPORT_VERSION,
            );
        }

        // Check exported_at field exists (informational, don't validate format)
        if !obj.contains_key("exported_at") {
            bail!("Missing 'exported_at' field");
        }

        // Check tables field is an object
        let tables = obj.get("tables")
            .and_then(|v| v.as_object())
            .context("Missing or invalid 'tables' field (must be an object)")?;

        // Verify all expected tables exist
        for table in TABLES {
            if !tables.contains_key(table) {
                bail!("Missing table '{}' in import data", table);
            }

            // Each table must be an array
            if !tables[table].is_array() {
                bail!("Table '{}' must be a JSON array", table);
            }
        }

        Ok(())
    }
}