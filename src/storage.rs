use rusqlite::{Connection, Result, params};
use std::path::Path;

#[derive(Debug)]
pub struct TxRecord {
    pub hash: String,
    pub payload: Vec<u8>,
    pub status: String,
    pub added_at: i64,
}

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS txs (
                hash TEXT PRIMARY KEY,
                payload BLOB,
                status TEXT,
                added_at INTEGER
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn insert_pending_tx(&mut self, hash: &str, payload: &[u8]) -> Result<()> {
        let tx = self.conn.transaction()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        tx.execute(
            "INSERT OR IGNORE INTO txs (hash, payload, status, added_at) VALUES (?1, ?2, 'pending', ?3)",
            params![hash, payload, now],
        )?;
        tx.commit()?;
        
        self.enforce_size_limit()?;
        Ok(())
    }

    pub fn update_status(&mut self, hash: &str, status: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE txs SET status = ?1 WHERE hash = ?2",
            params![status, hash],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_pending_txs(&self) -> Result<Vec<TxRecord>> {
        let mut stmt = self.conn.prepare("SELECT hash, payload, status, added_at FROM txs WHERE status = 'pending'")?;
        let rows = stmt.query_map([], |row| {
            Ok(TxRecord {
                hash: row.get(0)?,
                payload: row.get(1)?,
                status: row.get(2)?,
                added_at: row.get(3)?,
            })
        })?;

        let mut txs = Vec::new();
        for tx in rows {
            txs.push(tx?);
        }
        Ok(txs)
    }

    fn enforce_size_limit(&mut self) -> Result<()> {
        // SQLite page size * page count gives DB size
        let mut stmt = self.conn.prepare("PRAGMA page_count")?;
        let page_count: i64 = stmt.query_row([], |row| row.get(0))?;
        
        let mut stmt2 = self.conn.prepare("PRAGMA page_size")?;
        let page_size: i64 = stmt2.query_row([], |row| row.get(0))?;
        
        let size_bytes = page_count * page_size;
        let max_size = 50 * 1024 * 1024; // 50MB
        
        if size_bytes > max_size {
            // Delete oldest settled
            let tx = self.conn.transaction()?;
            tx.execute(
                "DELETE FROM txs WHERE hash IN (
                    SELECT hash FROM txs WHERE status = 'settled' ORDER BY added_at ASC LIMIT 100
                )",
                [],
            )?;
            tx.commit()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_schema_creation() {
        let mut storage = Storage::new(":memory:").unwrap();
        storage.insert_pending_tx("abc123hash", b"test payload").unwrap();
        
        let pending = storage.get_pending_txs().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].hash, "abc123hash");
        assert_eq!(pending[0].status, "pending");
        
        storage.update_status("abc123hash", "settled").unwrap();
        let pending_after = storage.get_pending_txs().unwrap();
        assert_eq!(pending_after.len(), 0);
    }
}
