use sqlx::{Error, PgConnection, PgPool, Postgres, Row};
use std::collections::{HashSet, VecDeque};

pub struct Rhyzome {
    pool: PgPool,
}

impl Rhyzome {
    pub async fn new(database_url: &str) -> Result<Rhyzome, Error> {
        let pool = PgPool::connect(database_url).await?;

        // Initialize types and tables if they don't exist
        pool.execute(
            "CREATE TABLE IF NOT EXISTS nodes (
                id TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
        )
        .await?;

        pool.execute(
            "CREATE TABLE IF NOT EXISTS relations (
                name TEXT,
                from_id TEXT,
                to_id TEXT
            )",
        )
        .await?;

        Ok(Rhyzome { pool })
    }

    pub async fn set(&self, id: &str, value: &str) -> Result<(), Error> {
        sqlx::query("INSERT INTO nodes (id, value) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET value = $2")
            .bind(id)
            .bind(value)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Option<String>, Error> {
        let row = sqlx::query("SELECT value FROM nodes WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.get("value")))
    }

    pub async fn delete(&self, id: &str) -> Result<(), Error> {
        sqlx::query("DELETE FROM nodes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn relate(&self, from_id: &str, relation_name: &str, to_id: &str) -> Result<(), Error> {
        sqlx::query("INSERT INTO relations (name, from_id, to_id) VALUES ($1, $2, $3)")
            .bind(relation_name)
            .bind(from_id)
            .bind(to_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_related(&self, id: &str, relation_name: &str) -> Result<Vec<String>, Error> {
        let rows = sqlx::query("SELECT to_id FROM relations WHERE from_id = $1 AND name = $2")
            .bind(id)
            .bind(relation_name)
            .fetch_all(&self.pool)
            .await?;

        let related_ids: Vec<String> = rows.iter().map(|r| r.get("to_id")).collect();
        Ok(related_ids)
    }

    pub async fn dfs(&self, start_id: &str) -> Result<Vec<String>, Error> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let mut result = Vec::new();

        visited.insert(start_id.to_owned());
        stack.push(start_id.to_owned());

        while let Some(id) = stack.pop() {
            result.push(id.clone());

            let related_ids = self.get_related(&id, "related").await?;
            for related_id in related_ids {
                if !visited.contains(&related_id) {
                    visited.insert(related_id.clone());
                    stack.push(related_id);
                }
            }
        }

        Ok(result)
    }

    pub async fn bfs(&self, start_id: &str) -> Result<Vec<String>, Error> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        visited.insert(start_id.to_owned());
        queue.push_back(start_id.to_owned());

        while let Some(id) = queue.pop_front() {
            result.push(id.clone());

            let related_ids = self.get_related(&id, "related").await?;
            for related_id in related_ids {
                if !visited.contains(&related_id) {
                    visited.insert(related_id.clone());
                    queue.push_back(related_id);
                }
            }
        }

        Ok(result)
    }

    pub async fn iter(&self) -> Result<Vec<String>, Error> {
        let rows = sqlx::query("SELECT id FROM nodes")
            .fetch_all(&self.pool)
            .await?;

        let ids: Vec<String> = rows.iter().map(|r| r.get("id")).collect();
        Ok(ids)
    }

    pub async fn query(&self, query: &str) -> Result<Vec<String>, Error> {
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await?;

        let results: Vec<String> = rows.iter().map(|r| r.get(0)).collect();
        Ok(results)
    }
}
