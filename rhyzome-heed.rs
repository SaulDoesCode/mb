use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use heed::{Env, Database, RwTxn};
use anyhow::{Result, anyhow};

pub struct Rhyzome {
    nodes_db: Database<String, Vec<u8>>,
    relations_db: Database<String, Vec<u8>>,
}

impl Rhyzome {
    pub fn new(db_path: &str) -> Result<Self> {
        let path = Path::new(db_path);
        let env = Env::new().open(path)?;

        let nodes_db: Database<String, Vec<u8>> = env.create_database(None)?;
        let relations_db: Database<String, Vec<u8>> = env.create_database(None)?;

        Ok(Self {
            nodes_db,
            relations_db,
        })
    }

    pub fn add_node(&self, id: &str, data: &[u8]) -> Result<()> {
        let txn = self.nodes_db.rw_txn()?;
        txn.put(&self.nodes_db, id, data)?;
        txn.commit()?;
        Ok(())
    }

    pub fn get_node(&self, id: &str) -> Result<Option<Vec<u8>>> {
        let ro_txn = self.nodes_db.ro_txn()?;
        let result = ro_txn.get(&self.nodes_db, &id)?;
        Ok(result)
    }

    pub fn remove_node(&self, id: &str) -> Result<()> {
        let mut txn = self.nodes_db.rw_txn()?;
        let result = txn.delete(&self.nodes_db, &id)?;
        txn.commit()?;
        if result {
            Ok(())
        } else {
            Err(anyhow!("Failed to remove node"))
        }
    }

    pub fn iter_nodes(&self) -> Result<Vec<String>> {
        let ro_txn = self.nodes_db.ro_txn()?;
        let iter = ro_txn.iter(&self.nodes_db)?;
        let ids: Vec<String> = iter.map(|(key, _)| key.to_owned()).collect();
        Ok(ids)
    }

    pub fn query_nodes<F>(&self, filter: F) -> Result<Vec<String>>
    where
        F: Fn(&[u8]) -> bool,
    {
        let ro_txn = self.nodes_db.ro_txn()?;
        let iter = ro_txn.iter(&self.nodes_db)?;
        let results: Vec<String> = iter
            .filter(|(_, value)| filter(value))
            .map(|(key, _)| key.to_owned())
            .collect();
        Ok(results)
    }

    pub fn add_relation(
        &self,
        relation_name: &str,
        node_id1: &str,
        node_id2: &str,
        data: &[u8],
    ) -> Result<()> {
        let relation_key = format!("{}_{}_{}", relation_name, node_id1, node_id2);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .to_be_bytes()
            .to_vec();

        let mut txn = self.relations_db.rw_txn()?;
        let mut existing_data = match txn.get(&self.relations_db, &relation_key)? {
            Some(mut data) => {
                data.extend(timestamp);
                data
            }
            None => timestamp,
        };

        existing_data.extend_from_slice(data);
        txn.put(&self.relations_db, &relation_key, &existing_data)?;
        txn.commit()?;
        Ok(())
    }

    pub fn get_relation(&self, relation_name: &str, node_id1: &str, node_id2: &str) -> Result<Option<Vec<u8>>> {
        let relation_key = format!("{}_{}_{}", relation_name, node_id1, node_id2);
        let ro_txn = self.relations_db.ro_txn()?;
        let result = ro_txn.get(&self.relations_db, &relation_key)?;
        Ok(result)
    }

    pub fn remove_relation(&self, relation_name: &str, node_id1: &str, node_id2: &str) -> Result<()> {
        let relation_key = format!("{}_{}_{}", relation_name, node_id1, node_id2);
        let mut txn = self.relations_db.rw_txn()?;
        let result = txn.delete(&self.relations_db, &relation_key)?;
        txn.commit()?;
        if result {
            Ok(())
        } else {
            Err(anyhow!("Failed to remove relation"))
        }
    }

    pub fn iter_relations(&self) -> Result<Vec<String>> {
        let ro_txn = self.relations_db.ro_txn()?;
        let iter = ro_txn.iter(&self.relations_db)?;
        let relation_names: Vec<String> = iter.map(|(key, _)| key.to_owned()).collect();
        Ok(relation_names)
    }

    pub fn query_relations<F>(&self, filter: F) -> Result<Vec<String>>
    where
        F: Fn(&[u8]) -> bool,
    {
        let ro_txn = self.relations_db.ro_txn()?;
        let iter = ro_txn.iter(&self.relations_db)?;
        let results: Vec<String> = iter
            .filter(|(_, value)| filter(value))
            .map(|(key, _)| key.to_owned())
            .collect();
        Ok(results)
    }

    pub fn dfs(&self, start_node_id: &str) -> Result<Vec<String>> {
        let mut visited: Vec<String> = Vec::new();
        let mut stack: Vec<String> = vec![start_node_id.to_string()];

        while let Some(node_id) = stack.pop() {
            if !visited.contains(&node_id) {
                visited.push(node_id.clone());

                let relations = self.query_relations(|value| {
                    let (name, id1, _) = split_relation_key(&value)?;
                    name.is_empty() && id1 == node_id
                })?;

                for relation in relations {
                    let (_, _, id2) = split_relation_key(&relation)?;
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

                let relations = self.query_relations(|value| {
                    let (name, id1, _) = split_relation_key(&value)?;
                    name.is_empty() && id1 == node_id
                })?;

                for relation in relations {
                    let (_, _, id2) = split_relation_key(&relation)?;
                    queue.push(id2);
                }
            }
        }

        Ok(visited)
    }
}

fn split_relation_key(relation_key: &str) -> Result<(&str, &str, &str)> {
    let parts: Vec<&str> = relation_key.split('_').collect();
    if parts.len() == 3 {
        Ok((parts[0], parts[1], parts[2]))
    } else {
        Err(anyhow!("Invalid relation key format"))
    }
}
