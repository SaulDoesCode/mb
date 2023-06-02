use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use heed::{EnvOpenOptions, Database, RwTxn, RoTxn, ByteSlice};
use heed::types::*;
use std::fs;
use std::path::Path;

pub struct Rhyzome {
    node_db: Database<Str, OwnedType<Node>>,
    relations_db: Database<Str, OwnedType<Relation>>,
    env: heed::Env,
}

impl Rhyzome {
    pub fn new() -> Result<Self> {
        fs::create_dir_all(Path::new("data").join("rhyzome.mdb")).context("Failed to create data directory")?;
        let env = EnvOpenOptions::new()
            .open(Path::new("data").join("rhyzome.mdb")).context("Failed to open heed environment")?;

        let node_db: Database<Str, OwnedType<Node>> = env.create_database(Some("node"))
            .context("Failed to create or open node database")?;

        let relations_db: Database<Str, OwnedType<Relation>> = env.create_database(Some("relations"))
            .context("Failed to create or open relations database")?;

        Ok(Rhyzome {
            node_db,
            relations_db,
            env,
        })
    }

    pub fn add_node(&self, node: Node) -> Result<()> {
        let mut txn = self.env.write_txn().context("Failed to begin write transaction")?;
        self.node_db.put(&mut txn, &node.id, &node).context("Failed to add node")?;
        txn.commit().context("Failed to commit transaction")?;
        Ok(())
    }

    pub fn get_node(&self, node_id: &str) -> Result<Option<Node>> {
        let ro_txn = self.env.read_txn().context("Failed to begin read transaction")?;
        let result = self.node_db.get(&ro_txn, &node_id).context("Failed to retrieve node")?;
        Ok(result)
    }

    pub fn update_node(&self, node: Node) -> Result<()> {
        let mut txn = self.env.write_txn().context("Failed to begin write transaction")?;
        self.node_db.put(&mut txn, &node.id, &node).context("Failed to update node")?;
        txn.commit().context("Failed to commit transaction")?;
        Ok(())
    }

    pub fn delete_node(&self, node_id: &str) -> Result<()> {
        let mut txn = self.env.write_txn().context("Failed to begin write transaction")?;
        self.node_db.delete(&mut txn, &node_id).context("Failed to delete node")?;
        txn.commit().context("Failed to commit transaction")?;
        Ok(())
    }
    
    pub fn iter_nodes(&self) -> Result<Vec<String>> {
        let ro_txn = self.env.read_txn().context("Failed to begin read transaction")?;
        let cursor = self.nodes_db.iter(&ro_txn)?;
        let mut result: Vec<String> = Vec::new();

        for res in cursor {
            let (node_key, _) = res?;
            let node_key_str = String::from_utf8(node_key.to_vec())
                .context("Failed to convert node key to String")?;
            result.push(node_key_str);
        }

        Ok(result)
    }
    
    pub fn query_nodes<F>(&self, filter: F) -> Result<Vec<String>>
    where
        F: Fn(&[u8]) -> bool,
    {
        let ro_txn = self.env.read_txn().context("Failed to begin read transaction")?;
        let cursor = self.nodes_db.iter(&ro_txn)?;
        let mut result: Vec<String> = Vec::new();

        for res in cursor {
            let (node_key, _) = res?;
            let node_key_bytes: &[u8] = &node_key;
            if filter(node_key_bytes) {
                let node_key_str = String::from_utf8(node_key_bytes.to_vec())
                    .context("Failed to convert node key to String")?;
                result.push(node_key_str);
            }
        }

        Ok(result)
    }

    
    pub fn add_relation(
        &self,
        relation_name: &str,
        node_id1: &str,
        node_id2: &str,
        relation: Relation,
    ) -> Result<()> {
        let relation_key = format!("{}_{}_{}", relation_name, node_id1, node_id2);
        let mut txn = self.env.write_txn().context("Failed to begin write transaction")?;
        self.relations_db.put(&mut txn, &relation_key, &relation).context("Failed to add relation")?;
        txn.commit().context("Failed to commit transaction")?;
        Ok(())
    }

    pub fn get_relation(
        &self,
        relation_name: &str,
        node_id1: &str,
        node_id2: &str,
    ) -> Result<Option<Relation>> {
        let relation_key = format!("{}_{}_{}", relation_name, node_id1, node_id2);
        let ro_txn = self.env.read_txn().context("Failed to begin read transaction")?;
        let result = self.relations_db.get(&ro_txn, &relation_key).context("Failed to retrieve relation")?;
        Ok(result)
    }

    pub fn update_relation(
        &self,
        relation_name: &str,
        node_id1: &str,
        node_id2: &str,
        relation: Relation,
    ) -> Result<()> {
        let relation_key = format!("{}_{}_{}", relation_name, node_id1, node_id2);
        let mut txn = self.env.write_txn().context("Failed to begin write transaction")?;
        self.relations_db.put(&mut txn, &relation_key, &relation).context("Failed to update relation")?;
        txn.commit().context("Failed to commit transaction")?;
        Ok(())
    }

    pub fn delete_relation(
        &self,
        relation_name: &str,
        node_id1: &str,
        node_id2: &str,
    ) -> Result<()> {
        let relation_key = format!("{}_{}_{}", relation_name, node_id1, node_id2);
        let mut txn = self.env.write_txn().context("Failed to begin write transaction")?;
        self.relations_db.delete(&mut txn, &relation_key).context("Failed to delete relation")?;
        txn.commit().context("Failed to commit transaction")?;
        Ok(())
    }

    pub fn get_related_nodes(
        &self,
        node_id: &str,
    ) -> Result<Vec<String>> {
        let relations = self.query_relations(|(_, id1, _)| id1 == node_id)?;
        let related_nodes: Vec<String> = relations.iter().map(|(_, _, id2)| id2.clone()).collect();
        Ok(related_nodes)
    }

    pub fn dfs(&self, start_node_id: &str) -> Result<Vec<String>> {
        let mut visited: Vec<String> = Vec::new();
        let mut stack: Vec<String> = vec![start_node_id.to_string()];

        while let Some(node_id) = stack.pop() {
            if !visited.contains(&node_id) {
                visited.push(node_id.clone());

                let relations = self.query_relations(|(_, id1, _)| id1 == &node_id)?;

                for (_, _, id2) in relations {
                    stack.push(id2);
                }
            }
        }

        Ok(visited)
    }

    pub fn bfs(&self, start_node_id: &str) -> Result<Vec<String>> {
        let mut visited: Vec<String> = Vec::new();
        let mut queue: Vec<String> = vec![start_node_id.to_string()];

        while let Some(node_id) = queue.pop(0) {
            if !visited.contains(&node_id) {
                visited.push(node_id.clone());

                let relations = self.query_relations(|(_, id1, _)| id1 == &node_id)?;

                for (_, _, id2) in relations {
                    queue.push(id2);
                }
            }
        }

        Ok(visited)
    }

    pub fn query_relations<F>(
        &self,
        filter: F,
    ) -> Result<Vec<(String, String, String)>>
    where
        F: FnMut(&(String, String, String)) -> bool,
    {
        let ro_txn = self.env.read_txn().context("Failed to begin read transaction")?;
        let cursor = self.relations_db.iter(&ro_txn)?;
        let mut result: Vec<(String, String, String)> = Vec::new();

        for res in cursor {
            let ((relation_key, relation), _) = res?;
            let (relation_name, id1, id2) = parse_relation_key(&relation_key)?;

            if filter(&(relation_name.clone(), id1.clone(), id2.clone())) {
                result.push((relation_name, id1, id2));
            }
        }

        Ok(result)
    }
    
    pub fn iter_relations(&self) -> Result<Vec<String>> {
        let ro_txn = self.env.read_txn().context("Failed to begin read transaction")?;
        let cursor = self.relations_db.iter(&ro_txn)?;
        let mut result: Vec<String> = Vec::new();

        for res in cursor {
            let ((relation_key, _), _) = res?;
            result.push(relation_key);
        }

        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub data: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relation {
    pub data: String,
    pub timestamp: DateTime<Utc>,
}

fn parse_relation_key(relation_key: &[u8]) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let relation_key = std::str::from_utf8(relation_key)?;
    let parts: Vec<&str> = relation_key.split('_').collect();
    if parts.len() != 3 {
        return Err("Invalid relation key".into());
    }
    Ok((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()))
}
