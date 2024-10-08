use openmls_traits::storage::*;
use serde::Serialize;
use std::io::Write;
use std::io::{stdout, Cursor};
use std::thread;
use std::{collections::HashMap, sync::RwLock};
use tokio::runtime::Runtime;

#[cfg(feature = "test-utils")]
use std::io::Write as _;
// use futures_util::StreamExt;

/// A storage for the V_TEST version.
#[cfg(any(test, feature = "test-utils"))]
mod test_store;

#[cfg(feature = "persistence")]
pub mod persistence;

#[macro_use]
extern crate anyhow;
extern crate async_trait;
extern crate log;
extern crate serde;
pub use sqlx;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::Row;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct LitePool {
    db: SqlitePool,
    tables: Tables,
}

impl LitePool {
    pub async fn new(db: SqlitePool, tables: Tables) -> anyhow::Result<LitePool> {
        tables.check()?;

        let this = Self { db, tables };
        this.migrate().await?;

        Ok(this)
    }

    /// try open tables
    pub async fn migrate(&self) -> anyhow::Result<()> {
        self.init().await?;
        Ok(())
    }

    /// https://docs.rs/sqlx-sqlite/0.7.1/sqlx_sqlite/struct.SqliteConnectOptions.html#impl-FromStr-for-SqliteConnectOptions
    pub async fn open(dbpath: &str, tables: Tables) -> anyhow::Result<LitePool> {
        let opts = dbpath
            .parse::<SqliteConnectOptions>()
            .expect("error in dbpath parse")
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            // prevent other thread open it
            .locking_mode(sqlx::sqlite::SqliteLockingMode::Exclusive)
            // or normal
            .synchronous(sqlx::sqlite::SqliteSynchronous::Full);

        log::trace!("SqlitePool open: {:?}", opts);
        let db = sqlx::sqlite::SqlitePoolOptions::new()
            // .max_connections(1)
            .connect_with(opts)
            .await
            .expect("error in connect_with");

        Self::new(db, tables).await
    }

    pub fn database(&self) -> &SqlitePool {
        &self.db
    }

    pub fn tables(&self) -> &Tables {
        &self.tables
    }

    pub async fn init(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.db)
            .await
            .map_err(|e| format_err!("run sqlite migrations failed: {}", e))?;

        Ok(())
    }

    #[inline]
    pub fn definition_identity(&self) -> &'static str {
        self.tables.identity
    }
}

impl Default for LitePool {
    fn default() -> Self {
        let opts = "/Users/shuyuandong/Desktop/openmls-openmls-v0.6.0-pre.3/sqlite_storage/mls-lite.sqlite"
            .parse::<SqliteConnectOptions>()
            .expect("error in dbpath parse")
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .locking_mode(sqlx::sqlite::SqliteLockingMode::Normal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

        log::trace!("SqlitePool open: {:?}", opts);
        let db = sqlx::sqlite::SqlitePoolOptions::new().connect_lazy_with(opts);

        Self {
            db,
            tables: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Tables {
    identity: &'static str,
}

impl Default for Tables {
    fn default() -> Self {
        Self {
            identity: "identity",
        }
    }
}

impl Tables {
    pub fn check(&self) -> anyhow::Result<()> {
        let strs = [self.identity];
        let mut names = strs.iter().filter(|s| !s.is_empty()).collect::<Vec<_>>();
        if names.len() != strs.len() {
            SqliteStorageError::InvalidArgument("empty table name".to_string());
        }

        names.dedup();
        if names.len() != strs.len() {
            SqliteStorageError::InvalidArgument("duplicate table name".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct SqliteStorage {
    username: String,
    pool: LitePool,
}

impl SqliteStorage {
    pub fn new(username: String) -> Self {
        Self {
            username,
            pool: Default::default(),
        }
    }

    pub fn test_write(&self) {
        // let key = b"alice".to_vec();
        // //[118, 97, 108, 117, 101]
        // let value = b"value".to_vec();
        let key = [
            123, 34, 118, 97, 108, 117, 101, 34, 58, 123, 34, 118, 101, 99, 34, 58, 91, 56, 52, 44,
            49, 48, 49, 44, 49, 49, 53, 44, 49, 49, 54, 44, 51, 50, 44, 55, 49, 44, 49, 49, 52, 44,
            49, 49, 49, 44, 49, 49, 55, 44, 49, 49, 50, 93, 125, 125,
        ]
        .to_vec();
        let value = [
            91, 91, 49, 49, 56, 44, 57, 55, 44, 49, 48, 56, 44, 49, 49, 55, 44, 49, 48, 49, 44, 52,
            57, 93, 44, 91, 49, 49, 56, 44, 57, 55, 44, 49, 48, 56, 44, 49, 49, 55, 44, 49, 48, 49,
            44, 53, 48, 93, 44, 91, 49, 49, 56, 44, 57, 55, 44, 49, 48, 56, 44, 49, 49, 55, 44, 49,
            48, 49, 44, 53, 48, 93, 44, 91, 49, 49, 56, 44, 57, 55, 44, 49, 48, 56, 44, 49, 49, 55,
            44, 49, 48, 49, 44, 53, 48, 93, 44, 91, 49, 49, 56, 44, 57, 55, 44, 49, 48, 56, 44, 49,
            49, 55, 44, 49, 48, 49, 44, 53, 48, 93, 44, 91, 49, 49, 56, 44, 57, 55, 44, 49, 48, 56,
            44, 49, 49, 55, 44, 49, 48, 49, 44, 53, 48, 93, 93,
        ]
        .to_vec();

        self.insert(&key, &value).unwrap();
        let val = self.get_value(&key).unwrap().unwrap();
        println!("{:?}", val);
    }

    pub fn test_append(&self) {
        // let key = b"bob".to_vec();
        // //[91, 91, 49, 49, 56, 44, 57, 55, 44, 49, 48, 56, 44, 49, 49, 55, 44, 49, 48, 49, 44, 52, 57, 93, 93]
        // let value = b"value2".to_vec();
        let key = [
            123, 34, 118, 97, 108, 117, 101, 34, 58, 123, 34, 118, 101, 99, 34, 58, 91, 56, 52, 44,
            49, 48, 49, 44, 49, 49, 53, 44, 49, 49, 54, 44, 51, 50, 44, 55, 49, 44, 49, 49, 52, 44,
            49, 49, 49, 44, 49, 49, 55, 44, 49, 49, 50, 93, 125, 125,
        ]
        .to_vec();
        let value = [
            91, 91, 49, 49, 56, 44, 57, 55, 44, 49, 48, 56, 44, 49, 49, 55, 44, 49, 48, 49, 44, 52,
            57, 93, 44, 91, 49, 49, 56, 44, 57, 55, 44, 49, 48, 56, 44, 49, 49, 55, 44, 49, 48, 49,
            44, 53, 48, 93, 44, 91, 49, 49, 56, 44, 57, 55, 44, 49, 48, 56, 44, 49, 49, 55, 44, 49,
            48, 49, 44, 53, 48, 93, 44, 91, 49, 49, 56, 44, 57, 55, 44, 49, 48, 56, 44, 49, 49, 55,
            44, 49, 48, 49, 44, 53, 48, 93, 44, 91, 49, 49, 56, 44, 57, 55, 44, 49, 48, 56, 44, 49,
            49, 55, 44, 49, 48, 49, 44, 53, 48, 93, 44, 91, 49, 49, 56, 44, 57, 55, 44, 49, 48, 56,
            44, 49, 49, 55, 44, 49, 48, 49, 44, 53, 48, 93, 93,
        ]
        .to_vec();

        // fetch value from db, falling back to an empty list if doens't exist
        let mut val = self.get_value(&key).expect("");
        let mut empty_vec = b"[]".to_vec();
        let list_bytes: &mut Vec<u8> = if val.is_none() {
            &mut empty_vec
        } else {
            val.as_mut().unwrap()
        };
        println!("list_bytes is {:?}", list_bytes);
        // parse old value and push new data
        let mut list: Vec<Vec<u8>> = serde_json::from_slice(list_bytes).unwrap();
        list.push(value);
        // write back, reusing the old buffer
        list_bytes.truncate(0);
        let mut cursor = Cursor::new(list_bytes);
        serde_json::to_writer(&mut cursor, &list).unwrap();
        self.insert(&key, cursor.into_inner())
            .expect("Insert error");

        let val = self.get_value(&key).unwrap().unwrap();
        println!("{:?}", val);
    }

    pub fn insert(&self, key: &Vec<u8>, value: &Vec<u8>) -> Result<(), String> {
        let sql = format!(
            "INSERT OR REPLACE INTO {} (user, iden_key, iden_value) values(?, ?, ?)",
            self.pool.definition_identity()
        );
        // stdout().write_all(b"insert start \n").unwrap();
        // stdout().write_all(format!("key {:?}\n", key.to_vec()).as_bytes()).unwrap();
        // stdout().write_all(format!("value {:?}\n", value.to_vec()).as_bytes()).unwrap();
        // {
        //     futures::executor::block_on(async move {
        //         sqlx::query(&sql)
        //             .bind(self.username.clone())
        //             .bind(key)
        //             .bind(value)
        //             .execute(&self.pool.db)
        //             .await
        //             .expect("execute insert error");
        //     });
        // }

        let (mp, mc) = flume::bounded(0);
        let db = self.pool.db.clone();
        let username = self.username.clone();
        let key = key.clone();
        let value = value.clone();
        let _res = tokio::spawn(async move {
            let sql = sqlx::query(&sql).bind(username).bind(key).bind(value);

            let r = sql.execute(&db).await;
            mp.send(r).expect("TODO: panic message");
        });
        mc.recv().unwrap().expect("TODO: panic message");

        // stdout().write_all(b"insert end\n").unwrap();
        Ok(())
    }

    pub fn del_value(&self, key: &Vec<u8>) -> Result<(), String> {
        let sql = format!(
            "DELETE from {} where user = ? and iden_key = ?",
            self.pool.definition_identity()
        );
        // {
        //     futures::executor::block_on(async move {
        //         sqlx::query(&sql)
        //             .bind(self.username.clone())
        //             .bind(key)
        //             .execute(&self.pool.db)
        //             .await
        //             .expect("execute insert error");
        //     });
        // }

        let (mp, mc) = flume::bounded(0);
        let db = self.pool.db.clone();
        let key = key.clone();
        let username = self.username.clone();
        let _res = tokio::spawn(async move {
            let sql = sqlx::query(&sql).bind(username).bind(key);

            let r = sql.execute(&db).await;
            mp.send(r).expect("TODO: panic message");
        });
        mc.recv().unwrap().expect("TODO: panic message");
        // stdout().write_all(b"delete end\n").unwrap();

        Ok(())
    }

    pub fn get_value(&self, key: &Vec<u8>) -> Result<Option<Vec<u8>>, String> {
        let sql = format!(
            "select iden_value from {} where user = ? and iden_key = ?",
            self.pool.definition_identity()
        );
        //
        // let value = futures::executor::block_on(async move {
        //     let result = sqlx::query(&sql)
        //         .bind(self.username.clone())
        //         .bind(key)
        //         .fetch_optional(&self.pool.db)
        //         .await
        //         .expect("get value error");
        //     result
        // });

        let (mp, mc) = flume::bounded(0);
        let db = self.pool.db.clone();
        let key = key.clone();
        let username = self.username.clone();
        let _res = tokio::spawn(async move {
            let sql = sqlx::query(&sql).bind(username).bind(key);

            let r = sql.fetch_optional(&db).await;
            mp.send(r).expect("TODO: panic message");
        });
        let value = mc.recv().unwrap().expect("TODO: panic message");

        if value.is_none() {
            return Ok(None);
        }
        let row = value.expect("get value error");
        let val = row.get::<'_, Vec<u8>, _>(0);
        Ok(Some(val))
    }

    /// Internal helper to abstract write operations.
    #[inline(always)]
    fn write<const VERSION: u16>(
        &self,
        label: &[u8],
        key: &[u8],
        value: Vec<u8>,
    ) -> Result<(), <Self as StorageProvider<CURRENT_VERSION>>::Error> {
        let storage_key = build_key_from_vec::<VERSION>(label, key.to_vec());

        #[cfg(feature = "test-utils")]
        log::debug!("  write key: {}", hex::encode(&storage_key));
        log::trace!("{}", std::backtrace::Backtrace::capture());
        // stdout().write_all(b"write\n").unwrap();
        self.insert(&storage_key, &value).expect("Insert error");
        Ok(())
    }

    fn append<const VERSION: u16>(
        &self,
        label: &[u8],
        key: &[u8],
        value: Vec<u8>,
    ) -> Result<(), <Self as StorageProvider<CURRENT_VERSION>>::Error> {
        let storage_key = build_key_from_vec::<VERSION>(label, key.to_vec());
        // stdout().write_all(b"append append append \n").unwrap();
        #[cfg(feature = "test-utils")]
        log::debug!("  write key: {}", hex::encode(&storage_key));
        log::trace!("{}", std::backtrace::Backtrace::capture());

        // fetch value from db, falling back to an empty list if doens't exist
        let mut val = self.get_value(&storage_key).expect("");
        let mut empty_vec = b"[]".to_vec();
        let list_bytes: &mut Vec<u8> = if val.is_none() {
            &mut empty_vec
        } else {
            val.as_mut().unwrap()
        };
        // parse old value and push new data
        let mut list: Vec<Vec<u8>> = serde_json::from_slice(list_bytes)?;
        list.push(value);
        // write back, reusing the old buffer
        list_bytes.truncate(0);
        let mut cursor = Cursor::new(list_bytes);
        serde_json::to_writer(&mut cursor, &list)?;
        self.insert(&storage_key, cursor.into_inner())
            .expect("Insert error");
        // stdout().write_all(b"append end\n").unwrap();
        Ok(())
    }

    fn remove_item<const VERSION: u16>(
        &self,
        label: &[u8],
        key: &[u8],
        value: Vec<u8>,
    ) -> Result<(), <Self as StorageProvider<CURRENT_VERSION>>::Error> {
        let storage_key = build_key_from_vec::<VERSION>(label, key.to_vec());

        #[cfg(feature = "test-utils")]
        log::debug!("  write key: {}", hex::encode(&storage_key));
        log::trace!("{}", std::backtrace::Backtrace::capture());

        let mut val = self.get_value(&storage_key).expect("");
        let mut empty_vec = b"[]".to_vec();
        let list_bytes: &mut Vec<u8> = if val.is_none() {
            &mut empty_vec
        } else {
            val.as_mut().unwrap()
        };
        // parse old value, find value to delete and remove it from list
        let mut list: Vec<Vec<u8>> = serde_json::from_slice(&*list_bytes)?;
        if let Some(pos) = list.iter().position(|stored_item| stored_item == &value) {
            list.remove(pos);
        }
        // write back, reusing the old buffer
        list_bytes.truncate(0);
        let mut cursor = Cursor::new(list_bytes);
        serde_json::to_writer(&mut cursor, &list)?;

        self.insert(&storage_key, cursor.into_inner())
            .expect("Insert error");
        stdout().write_all(b"remove end\n").unwrap();
        Ok(())
    }

    /// Internal helper to abstract read operations.
    #[inline(always)]
    fn read<const VERSION: u16, V: Entity<VERSION>>(
        &self,
        label: &[u8],
        key: &[u8],
    ) -> Result<Option<V>, <Self as StorageProvider<CURRENT_VERSION>>::Error> {
        let storage_key = build_key_from_vec::<VERSION>(label, key.to_vec());
        // println!("read");
        #[cfg(feature = "test-utils")]
        log::debug!("  read key: {}", hex::encode(&storage_key));
        log::trace!("{}", std::backtrace::Backtrace::capture());

        let value = self.get_value(&storage_key).expect("");

        if let Some(value) = &value {
            serde_json::from_slice(value)
                .map_err(|_| SqliteStorageError::SerializationError)
                .map(|v| Some(v))
        } else {
            Ok(None)
        }
    }

    /// Internal helper to abstract read operations.
    #[inline(always)]
    fn read_list<const VERSION: u16, V: Entity<VERSION>>(
        &self,
        label: &[u8],
        key: &[u8],
    ) -> Result<Vec<V>, <Self as StorageProvider<CURRENT_VERSION>>::Error> {
        let mut storage_key = label.to_vec();
        storage_key.extend_from_slice(key);
        storage_key.extend_from_slice(&u16::to_be_bytes(VERSION));
        #[cfg(feature = "test-utils")]
        log::debug!("  read list key: {}", hex::encode(&storage_key));
        log::trace!("{}", std::backtrace::Backtrace::capture());

        let value: Vec<Vec<u8>> = match self.get_value(&storage_key).expect("") {
            Some(list_bytes) => serde_json::from_slice(&*list_bytes).unwrap(),
            None => vec![],
        };

        value
            .iter()
            .map(|value_bytes| serde_json::from_slice(value_bytes))
            .collect::<Result<Vec<V>, _>>()
            .map_err(|_| SqliteStorageError::SerializationError)
    }

    /// Internal helper to abstract delete operations.
    #[inline(always)]
    fn delete<const VERSION: u16>(
        &self,
        label: &[u8],
        key: &[u8],
    ) -> Result<(), <Self as StorageProvider<CURRENT_VERSION>>::Error> {
        let mut storage_key = label.to_vec();
        storage_key.extend_from_slice(key);
        storage_key.extend_from_slice(&u16::to_be_bytes(VERSION));

        #[cfg(feature = "test-utils")]
        log::debug!("  delete key: {}", hex::encode(&storage_key));
        log::trace!("{}", std::backtrace::Backtrace::capture());

        self.del_value(&storage_key).expect("TODO: panic message");

        Ok(())
    }
}

/// Errors thrown by the key store.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum SqliteStorageError {
    #[error("Error InvalidArgument value.")]
    InvalidArgument(String),
    #[error("The key store does not allow storing serialized values.")]
    UnsupportedValueTypeBytes,
    #[error("Updating is not supported by this key store.")]
    UnsupportedMethod,
    #[error("Error serializing value.")]
    SerializationError,
    #[error("Value does not exist.")]
    None,
}

const KEY_PACKAGE_LABEL: &[u8] = b"KeyPackage";
const PSK_LABEL: &[u8] = b"Psk";
const ENCRYPTION_KEY_PAIR_LABEL: &[u8] = b"EncryptionKeyPair";
const SIGNATURE_KEY_PAIR_LABEL: &[u8] = b"SignatureKeyPair";
const EPOCH_KEY_PAIRS_LABEL: &[u8] = b"EpochKeyPairs";

// related to PublicGroup
const TREE_LABEL: &[u8] = b"Tree";
const GROUP_CONTEXT_LABEL: &[u8] = b"GroupContext";
const INTERIM_TRANSCRIPT_HASH_LABEL: &[u8] = b"InterimTranscriptHash";
const CONFIRMATION_TAG_LABEL: &[u8] = b"ConfirmationTag";

// related to CoreGroup
const OWN_LEAF_NODE_INDEX_LABEL: &[u8] = b"OwnLeafNodeIndex";
const EPOCH_SECRETS_LABEL: &[u8] = b"EpochSecrets";
const RESUMPTION_PSK_STORE_LABEL: &[u8] = b"ResumptionPsk";
const MESSAGE_SECRETS_LABEL: &[u8] = b"MessageSecrets";

// related to MlsGroup
const JOIN_CONFIG_LABEL: &[u8] = b"MlsGroupJoinConfig";
const OWN_LEAF_NODES_LABEL: &[u8] = b"OwnLeafNodes";
const GROUP_STATE_LABEL: &[u8] = b"GroupState";
const QUEUED_PROPOSAL_LABEL: &[u8] = b"QueuedProposal";
const PROPOSAL_QUEUE_REFS_LABEL: &[u8] = b"ProposalQueueRefs";

impl StorageProvider<CURRENT_VERSION> for SqliteStorage {
    type Error = SqliteStorageError;

    fn write_mls_join_config<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        MlsGroupJoinConfig: traits::MlsGroupJoinConfig<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        config: &MlsGroupJoinConfig,
    ) -> Result<(), Self::Error> {
        let key = serde_json::to_vec(group_id).unwrap();
        let value = serde_json::to_vec(config).unwrap();
        // stdout().write_all(b" >>> write_mls_join_config :)\n").unwrap();
        self.write::<CURRENT_VERSION>(JOIN_CONFIG_LABEL, &key, value)
    }

    fn append_own_leaf_node<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        LeafNode: traits::LeafNode<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        leaf_node: &LeafNode,
    ) -> Result<(), Self::Error> {
        // stdout().write_all(b"append_own_leaf_node \n").unwrap();
        let key = serde_json::to_vec(group_id)?;
        let value = serde_json::to_vec(leaf_node)?;
        self.append::<CURRENT_VERSION>(OWN_LEAF_NODES_LABEL, &key, value)
    }

    fn queue_proposal<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        ProposalRef: traits::ProposalRef<CURRENT_VERSION>,
        QueuedProposal: traits::QueuedProposal<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        proposal_ref: &ProposalRef,
        proposal: &QueuedProposal,
    ) -> Result<(), Self::Error> {
        // write proposal to key (group_id, proposal_ref)
        let key = serde_json::to_vec(&(group_id, proposal_ref))?;
        let value = serde_json::to_vec(proposal)?;
        self.write::<CURRENT_VERSION>(QUEUED_PROPOSAL_LABEL, &key, value)?;

        // update proposal list for group_id
        let key = serde_json::to_vec(group_id)?;
        let value = serde_json::to_vec(proposal_ref)?;
        self.append::<CURRENT_VERSION>(PROPOSAL_QUEUE_REFS_LABEL, &key, value)?;

        Ok(())
    }

    fn write_tree<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        TreeSync: traits::TreeSync<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        tree: &TreeSync,
    ) -> Result<(), Self::Error> {
        self.write::<CURRENT_VERSION>(
            TREE_LABEL,
            &serde_json::to_vec(&group_id).unwrap(),
            serde_json::to_vec(&tree).unwrap(),
        )
    }

    fn write_interim_transcript_hash<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        InterimTranscriptHash: traits::InterimTranscriptHash<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        interim_transcript_hash: &InterimTranscriptHash,
    ) -> Result<(), Self::Error> {
        let key = build_key::<CURRENT_VERSION, &GroupId>(INTERIM_TRANSCRIPT_HASH_LABEL, group_id);
        let value = serde_json::to_vec(&interim_transcript_hash).unwrap();

        self.insert(&key, &value).expect("TODO: panic message");
        Ok(())
    }

    fn write_context<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        GroupContext: traits::GroupContext<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        group_context: &GroupContext,
    ) -> Result<(), Self::Error> {
        let key = build_key::<CURRENT_VERSION, &GroupId>(GROUP_CONTEXT_LABEL, group_id);
        let value = serde_json::to_vec(&group_context).unwrap();

        self.insert(&key, &value).expect("TODO: panic message");
        Ok(())
    }

    fn write_confirmation_tag<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        ConfirmationTag: traits::ConfirmationTag<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        confirmation_tag: &ConfirmationTag,
    ) -> Result<(), Self::Error> {
        let key = build_key::<CURRENT_VERSION, &GroupId>(CONFIRMATION_TAG_LABEL, group_id);
        let value = serde_json::to_vec(&confirmation_tag).unwrap();

        self.insert(&key, &value).expect("TODO: panic message");
        Ok(())
    }

    fn write_group_state<
        GroupState: traits::GroupState<CURRENT_VERSION>,
        GroupId: traits::GroupId<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        group_state: &GroupState,
    ) -> Result<(), Self::Error> {
        // stdout().write_all(b"write_group_state\n").unwrap();
        self.write::<CURRENT_VERSION>(
            GROUP_STATE_LABEL,
            &serde_json::to_vec(group_id)?,
            serde_json::to_vec(group_state)?,
        )
    }

    fn write_message_secrets<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        MessageSecrets: traits::MessageSecrets<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        message_secrets: &MessageSecrets,
    ) -> Result<(), Self::Error> {
        self.write::<CURRENT_VERSION>(
            MESSAGE_SECRETS_LABEL,
            &serde_json::to_vec(group_id)?,
            serde_json::to_vec(message_secrets)?,
        )
    }

    fn write_resumption_psk_store<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        ResumptionPskStore: traits::ResumptionPskStore<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        resumption_psk_store: &ResumptionPskStore,
    ) -> Result<(), Self::Error> {
        self.write::<CURRENT_VERSION>(
            RESUMPTION_PSK_STORE_LABEL,
            &serde_json::to_vec(group_id)?,
            serde_json::to_vec(resumption_psk_store)?,
        )
    }

    fn write_own_leaf_index<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        LeafNodeIndex: traits::LeafNodeIndex<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        own_leaf_index: &LeafNodeIndex,
    ) -> Result<(), Self::Error> {
        self.write::<CURRENT_VERSION>(
            OWN_LEAF_NODE_INDEX_LABEL,
            &serde_json::to_vec(group_id)?,
            serde_json::to_vec(own_leaf_index)?,
        )
    }

    fn write_group_epoch_secrets<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        GroupEpochSecrets: traits::GroupEpochSecrets<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        group_epoch_secrets: &GroupEpochSecrets,
    ) -> Result<(), Self::Error> {
        self.write::<CURRENT_VERSION>(
            EPOCH_SECRETS_LABEL,
            &serde_json::to_vec(group_id)?,
            serde_json::to_vec(group_epoch_secrets)?,
        )
    }

    fn write_signature_key_pair<
        SignaturePublicKey: traits::SignaturePublicKey<CURRENT_VERSION>,
        SignatureKeyPair: traits::SignatureKeyPair<CURRENT_VERSION>,
    >(
        &self,
        public_key: &SignaturePublicKey,
        signature_key_pair: &SignatureKeyPair,
    ) -> Result<(), Self::Error> {
        let key =
            build_key::<CURRENT_VERSION, &SignaturePublicKey>(SIGNATURE_KEY_PAIR_LABEL, public_key);
        let value = serde_json::to_vec(&signature_key_pair).unwrap();

        self.insert(&key, &value).expect("TODO: panic message");
        Ok(())
    }

    fn write_encryption_key_pair<
        EncryptionKey: traits::EncryptionKey<CURRENT_VERSION>,
        HpkeKeyPair: traits::HpkeKeyPair<CURRENT_VERSION>,
    >(
        &self,
        public_key: &EncryptionKey,
        key_pair: &HpkeKeyPair,
    ) -> Result<(), Self::Error> {
        self.write::<CURRENT_VERSION>(
            ENCRYPTION_KEY_PAIR_LABEL,
            &serde_json::to_vec(public_key).unwrap(),
            serde_json::to_vec(key_pair).unwrap(),
        )
    }

    fn write_encryption_epoch_key_pairs<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        EpochKey: traits::EpochKey<CURRENT_VERSION>,
        HpkeKeyPair: traits::HpkeKeyPair<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        epoch: &EpochKey,
        leaf_index: u32,
        key_pairs: &[HpkeKeyPair],
    ) -> Result<(), Self::Error> {
        let key = epoch_key_pairs_id(group_id, epoch, leaf_index)?;
        let value = serde_json::to_vec(key_pairs)?;
        log::debug!("Writing encryption epoch key pairs");
        #[cfg(feature = "test-utils")]
        {
            log::debug!("  key: {}", hex::encode(&key));
            log::debug!("  value: {}", hex::encode(&value));
        }

        self.write::<CURRENT_VERSION>(EPOCH_KEY_PAIRS_LABEL, &key, value)
    }

    fn write_key_package<
        HashReference: traits::HashReference<CURRENT_VERSION>,
        KeyPackage: traits::KeyPackage<CURRENT_VERSION>,
    >(
        &self,
        hash_ref: &HashReference,
        key_package: &KeyPackage,
    ) -> Result<(), Self::Error> {
        let key = serde_json::to_vec(&hash_ref).unwrap();
        let value = serde_json::to_vec(&key_package).unwrap();

        self.write::<CURRENT_VERSION>(KEY_PACKAGE_LABEL, &key, value)
            .unwrap();

        Ok(())
    }

    fn write_psk<
        PskId: traits::PskId<CURRENT_VERSION>,
        PskBundle: traits::PskBundle<CURRENT_VERSION>,
    >(
        &self,
        psk_id: &PskId,
        psk: &PskBundle,
    ) -> Result<(), Self::Error> {
        self.write::<CURRENT_VERSION>(
            PSK_LABEL,
            &serde_json::to_vec(&psk_id).unwrap(),
            serde_json::to_vec(&psk).unwrap(),
        )
    }

    fn mls_group_join_config<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        MlsGroupJoinConfig: traits::MlsGroupJoinConfig<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<MlsGroupJoinConfig>, Self::Error> {
        self.read(JOIN_CONFIG_LABEL, &serde_json::to_vec(group_id).unwrap())
    }

    fn own_leaf_nodes<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        LeafNode: traits::LeafNode<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Vec<LeafNode>, Self::Error> {
        self.read_list(OWN_LEAF_NODES_LABEL, &serde_json::to_vec(group_id).unwrap())
    }

    fn queued_proposal_refs<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        ProposalRef: traits::ProposalRef<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Vec<ProposalRef>, Self::Error> {
        self.read_list(PROPOSAL_QUEUE_REFS_LABEL, &serde_json::to_vec(group_id)?)
    }

    fn queued_proposals<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        ProposalRef: traits::ProposalRef<CURRENT_VERSION>,
        QueuedProposal: traits::QueuedProposal<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Vec<(ProposalRef, QueuedProposal)>, Self::Error> {
        let refs: Vec<ProposalRef> =
            self.read_list(PROPOSAL_QUEUE_REFS_LABEL, &serde_json::to_vec(group_id)?)?;

        refs.into_iter()
            .map(|proposal_ref| -> Result<_, _> {
                let key = (group_id, &proposal_ref);
                let key = serde_json::to_vec(&key)?;

                let proposal = self.read(QUEUED_PROPOSAL_LABEL, &key)?.unwrap();
                Ok((proposal_ref, proposal))
            })
            .collect::<Result<Vec<_>, _>>()
    }

    fn tree<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        TreeSync: traits::TreeSync<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<TreeSync>, Self::Error> {
        let key = build_key::<CURRENT_VERSION, &GroupId>(TREE_LABEL, group_id);

        let Some(value) = &self.get_value(&key).expect("") else {
            return Ok(None);
        };
        let value = serde_json::from_slice(value).unwrap();

        Ok(value)
    }

    fn group_context<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        GroupContext: traits::GroupContext<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<GroupContext>, Self::Error> {
        let key = build_key::<CURRENT_VERSION, &GroupId>(GROUP_CONTEXT_LABEL, group_id);

        let Some(value) = &self.get_value(&key).expect("") else {
            return Ok(None);
        };
        let value = serde_json::from_slice(value).unwrap();

        Ok(value)
    }

    fn interim_transcript_hash<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        InterimTranscriptHash: traits::InterimTranscriptHash<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<InterimTranscriptHash>, Self::Error> {
        let key = build_key::<CURRENT_VERSION, &GroupId>(INTERIM_TRANSCRIPT_HASH_LABEL, group_id);

        let Some(value) = &self.get_value(&key).expect("") else {
            return Ok(None);
        };
        let value = serde_json::from_slice(value).unwrap();

        Ok(value)
    }

    fn confirmation_tag<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        ConfirmationTag: traits::ConfirmationTag<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<ConfirmationTag>, Self::Error> {
        let key = build_key::<CURRENT_VERSION, &GroupId>(CONFIRMATION_TAG_LABEL, group_id);

        let Some(value) = &self.get_value(&key).expect("") else {
            return Ok(None);
        };
        let value = serde_json::from_slice(value).unwrap();

        Ok(value)
    }

    fn group_state<
        GroupState: traits::GroupState<CURRENT_VERSION>,
        GroupId: traits::GroupId<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<GroupState>, Self::Error> {
        self.read(GROUP_STATE_LABEL, &serde_json::to_vec(&group_id)?)
    }

    fn message_secrets<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        MessageSecrets: traits::MessageSecrets<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<MessageSecrets>, Self::Error> {
        self.read(MESSAGE_SECRETS_LABEL, &serde_json::to_vec(group_id)?)
    }

    fn resumption_psk_store<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        ResumptionPskStore: traits::ResumptionPskStore<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<ResumptionPskStore>, Self::Error> {
        self.read(RESUMPTION_PSK_STORE_LABEL, &serde_json::to_vec(group_id)?)
    }

    fn own_leaf_index<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        LeafNodeIndex: traits::LeafNodeIndex<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<LeafNodeIndex>, Self::Error> {
        self.read(OWN_LEAF_NODE_INDEX_LABEL, &serde_json::to_vec(group_id)?)
    }

    fn group_epoch_secrets<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        GroupEpochSecrets: traits::GroupEpochSecrets<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<GroupEpochSecrets>, Self::Error> {
        self.read(EPOCH_SECRETS_LABEL, &serde_json::to_vec(group_id)?)
    }

    fn signature_key_pair<
        SignaturePublicKey: traits::SignaturePublicKey<CURRENT_VERSION>,
        SignatureKeyPair: traits::SignatureKeyPair<CURRENT_VERSION>,
    >(
        &self,
        public_key: &SignaturePublicKey,
    ) -> Result<Option<SignatureKeyPair>, Self::Error> {
        let key =
            build_key::<CURRENT_VERSION, &SignaturePublicKey>(SIGNATURE_KEY_PAIR_LABEL, public_key);

        let Some(value) = &self.get_value(&key).expect("") else {
            return Ok(None);
        };
        let value = serde_json::from_slice(value).unwrap();

        Ok(value)
    }

    fn encryption_key_pair<
        HpkeKeyPair: traits::HpkeKeyPair<CURRENT_VERSION>,
        EncryptionKey: traits::EncryptionKey<CURRENT_VERSION>,
    >(
        &self,
        public_key: &EncryptionKey,
    ) -> Result<Option<HpkeKeyPair>, Self::Error> {
        self.read(
            ENCRYPTION_KEY_PAIR_LABEL,
            &serde_json::to_vec(public_key).unwrap(),
        )
    }

    fn encryption_epoch_key_pairs<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        EpochKey: traits::EpochKey<CURRENT_VERSION>,
        HpkeKeyPair: traits::HpkeKeyPair<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        epoch: &EpochKey,
        leaf_index: u32,
    ) -> Result<Vec<HpkeKeyPair>, Self::Error> {
        let key = epoch_key_pairs_id(group_id, epoch, leaf_index)?;
        let storage_key = build_key_from_vec::<CURRENT_VERSION>(EPOCH_KEY_PAIRS_LABEL, key);
        log::debug!("Reading encryption epoch key pairs");

        let value = self.get_value(&storage_key).expect("");

        #[cfg(feature = "test-utils")]
        log::debug!("  key: {}", hex::encode(&storage_key));

        if let Some(value) = &value {
            #[cfg(feature = "test-utils")]
            log::debug!("  value: {}", hex::encode(value));
            return Ok(serde_json::from_slice(value).unwrap());
        }

        Err(SqliteStorageError::None)
    }

    fn key_package<
        KeyPackageRef: traits::HashReference<CURRENT_VERSION>,
        KeyPackage: traits::KeyPackage<CURRENT_VERSION>,
    >(
        &self,
        hash_ref: &KeyPackageRef,
    ) -> Result<Option<KeyPackage>, Self::Error> {
        let key = serde_json::to_vec(&hash_ref).unwrap();
        self.read(KEY_PACKAGE_LABEL, &key)
    }

    fn psk<PskBundle: traits::PskBundle<CURRENT_VERSION>, PskId: traits::PskId<CURRENT_VERSION>>(
        &self,
        psk_id: &PskId,
    ) -> Result<Option<PskBundle>, Self::Error> {
        self.read(PSK_LABEL, &serde_json::to_vec(&psk_id).unwrap())
    }

    fn remove_proposal<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        ProposalRef: traits::ProposalRef<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        proposal_ref: &ProposalRef,
    ) -> Result<(), Self::Error> {
        let key = serde_json::to_vec(group_id).unwrap();
        let value = serde_json::to_vec(proposal_ref).unwrap();

        self.remove_item::<CURRENT_VERSION>(PROPOSAL_QUEUE_REFS_LABEL, &key, value)?;

        let key = serde_json::to_vec(&(group_id, proposal_ref)).unwrap();
        self.delete::<CURRENT_VERSION>(QUEUED_PROPOSAL_LABEL, &key)
    }

    fn delete_own_leaf_nodes<GroupId: traits::GroupId<CURRENT_VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(OWN_LEAF_NODES_LABEL, &serde_json::to_vec(group_id).unwrap())
    }

    fn delete_group_config<GroupId: traits::GroupId<CURRENT_VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(JOIN_CONFIG_LABEL, &serde_json::to_vec(group_id).unwrap())
    }

    fn delete_tree<GroupId: traits::GroupId<CURRENT_VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(TREE_LABEL, &serde_json::to_vec(group_id).unwrap())
    }

    fn delete_confirmation_tag<GroupId: traits::GroupId<CURRENT_VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(
            CONFIRMATION_TAG_LABEL,
            &serde_json::to_vec(group_id).unwrap(),
        )
    }

    fn delete_group_state<GroupId: traits::GroupId<CURRENT_VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(GROUP_STATE_LABEL, &serde_json::to_vec(group_id)?)
    }

    fn delete_context<GroupId: traits::GroupId<CURRENT_VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(GROUP_CONTEXT_LABEL, &serde_json::to_vec(group_id).unwrap())
    }

    fn delete_interim_transcript_hash<GroupId: traits::GroupId<CURRENT_VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(
            INTERIM_TRANSCRIPT_HASH_LABEL,
            &serde_json::to_vec(group_id).unwrap(),
        )
    }

    fn delete_message_secrets<GroupId: traits::GroupId<CURRENT_VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(MESSAGE_SECRETS_LABEL, &serde_json::to_vec(group_id)?)
    }

    fn delete_all_resumption_psk_secrets<GroupId: traits::GroupId<CURRENT_VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(RESUMPTION_PSK_STORE_LABEL, &serde_json::to_vec(group_id)?)
    }

    fn delete_own_leaf_index<GroupId: traits::GroupId<CURRENT_VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(OWN_LEAF_NODE_INDEX_LABEL, &serde_json::to_vec(group_id)?)
    }

    fn delete_group_epoch_secrets<GroupId: traits::GroupId<CURRENT_VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(EPOCH_SECRETS_LABEL, &serde_json::to_vec(group_id)?)
    }

    fn clear_proposal_queue<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        ProposalRef: traits::ProposalRef<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        // Get all proposal refs for this group.
        let proposal_refs: Vec<ProposalRef> =
            self.read_list(PROPOSAL_QUEUE_REFS_LABEL, &serde_json::to_vec(group_id)?)?;
        for proposal_ref in proposal_refs {
            // Delete all proposals.
            let key = serde_json::to_vec(&(group_id, proposal_ref))?;
            self.del_value(&key).unwrap()
        }

        // Delete the proposal refs from the store.
        let key = build_key::<CURRENT_VERSION, &GroupId>(PROPOSAL_QUEUE_REFS_LABEL, group_id);
        self.del_value(&key).unwrap();

        Ok(())
    }

    fn delete_signature_key_pair<
        SignaturePublicKeuy: traits::SignaturePublicKey<CURRENT_VERSION>,
    >(
        &self,
        public_key: &SignaturePublicKeuy,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(
            SIGNATURE_KEY_PAIR_LABEL,
            &serde_json::to_vec(public_key).unwrap(),
        )
    }

    fn delete_encryption_key_pair<EncryptionKey: traits::EncryptionKey<CURRENT_VERSION>>(
        &self,
        public_key: &EncryptionKey,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(
            ENCRYPTION_KEY_PAIR_LABEL,
            &serde_json::to_vec(&public_key).unwrap(),
        )
    }

    fn delete_encryption_epoch_key_pairs<
        GroupId: traits::GroupId<CURRENT_VERSION>,
        EpochKey: traits::EpochKey<CURRENT_VERSION>,
    >(
        &self,
        group_id: &GroupId,
        epoch: &EpochKey,
        leaf_index: u32,
    ) -> Result<(), Self::Error> {
        let key = epoch_key_pairs_id(group_id, epoch, leaf_index)?;
        self.delete::<CURRENT_VERSION>(EPOCH_KEY_PAIRS_LABEL, &key)
    }

    fn delete_key_package<KeyPackageRef: traits::HashReference<CURRENT_VERSION>>(
        &self,
        hash_ref: &KeyPackageRef,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(KEY_PACKAGE_LABEL, &serde_json::to_vec(&hash_ref)?)
    }

    fn delete_psk<PskKey: traits::PskId<CURRENT_VERSION>>(
        &self,
        psk_id: &PskKey,
    ) -> Result<(), Self::Error> {
        self.delete::<CURRENT_VERSION>(PSK_LABEL, &serde_json::to_vec(&psk_id)?)
    }
}

/// Build a key with version and label.
fn build_key_from_vec<const V: u16>(label: &[u8], key: Vec<u8>) -> Vec<u8> {
    let mut key_out = label.to_vec();
    key_out.extend_from_slice(&key);
    key_out.extend_from_slice(&u16::to_be_bytes(V));
    key_out
}

/// Build a key with version and label.
fn build_key<const V: u16, K: Serialize>(label: &[u8], key: K) -> Vec<u8> {
    build_key_from_vec::<V>(label, serde_json::to_vec(&key).unwrap())
}

fn epoch_key_pairs_id(
    group_id: &impl traits::GroupId<CURRENT_VERSION>,
    epoch: &impl traits::EpochKey<CURRENT_VERSION>,
    leaf_index: u32,
) -> Result<Vec<u8>, <SqliteStorage as StorageProvider<CURRENT_VERSION>>::Error> {
    let mut key = serde_json::to_vec(group_id)?;
    key.extend_from_slice(&serde_json::to_vec(epoch)?);
    key.extend_from_slice(&serde_json::to_vec(&leaf_index)?);
    Ok(key)
}

impl From<serde_json::Error> for SqliteStorageError {
    fn from(_: serde_json::Error) -> Self {
        Self::SerializationError
    }
}
