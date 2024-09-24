use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize)]
struct SerializableKeyStore {
    values: HashMap<String, String>,
}

impl super::SqliteStorage {
    pub fn save(&self) -> Result<(), String> {
        // for (key, value) in &*self.values.read().unwrap() {
        //    self.insert(&key, &value).expect("");
        // }
        Ok(())
    }

    pub async fn load(&mut self) -> Result<(), String> {
        // let sql = format!("select iden_key, iden_value from {}", self.pool.definition_identity());
        // let mut idens = sqlx::query(&sql)
        //     .fetch(&self.pool.db);
        // let mut ks_map = self.values.write().unwrap();
        // while let Some(iden) = idens.next().await {
        //     let iden = iden.expect("get iden error");
        //     let key: Vec<u8> = iden.get::<'_, Vec<u8>, _>(0);
        //     let value: Vec<u8> = iden.get::<'_, Vec<u8>, _>(1);
        //     ks_map.insert(key, value);
        // }
        Ok(())
    }
}
