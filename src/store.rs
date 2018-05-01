use bitcoin::blockdata::block::BlockHeader;
use bitcoin::network::serialize::deserialize;
use rocksdb;

use types::Bytes;

pub struct Store {
    db: rocksdb::DB,
}

pub struct Row {
    pub key: Bytes,
    pub value: Bytes,
}

#[derive(Debug)]
pub struct StoreOptions {
    pub auto_compact: bool,
}

impl Store {
    /// Opens a new RocksDB at the specified location.
    pub fn open(path: &str, opts: StoreOptions) -> Store {
        info!("opening {} with {:?}", path, &opts);
        let mut db_opts = rocksdb::Options::default();
        db_opts.create_if_missing(true);
        db_opts.set_compaction_style(rocksdb::DBCompactionStyle::Level);
        db_opts.set_use_fsync(false);
        db_opts.set_compression_type(rocksdb::DBCompressionType::Snappy);
        db_opts.set_target_file_size_base(64 << 20);
        db_opts.set_write_buffer_size(64 << 20);
        db_opts.set_disable_auto_compactions(!opts.auto_compact);

        let mut block_opts = rocksdb::BlockBasedOptions::default();
        block_opts.set_block_size(256 << 10);
        Store {
            db: rocksdb::DB::open(&db_opts, &path).unwrap(),
        }
    }

    pub fn persist(&self, rows: &Vec<Row>) {
        let mut batch = rocksdb::WriteBatch::default();
        for row in rows {
            batch.put(row.key.as_slice(), row.value.as_slice()).unwrap();
        }
        let mut opts = rocksdb::WriteOptions::new();
        opts.set_sync(true);
        self.db.write_opt(batch, &opts).unwrap();
    }

    pub fn read_header(&self, blockhash: &[u8]) -> Option<BlockHeader> {
        self.get(&[b"B", blockhash].concat())
            .map(|value| deserialize(&value).unwrap())
    }

    pub fn compact_if_needed(&self) {
        let key = b"F"; // full compaction marker
        if self.get(key).is_some() {
            return;
        }
        info!("full compaction");
        self.db.compact_range(None, None); // should take a while
        self.db.put(key, b"").unwrap();
    }

    pub fn get(&self, key: &[u8]) -> Option<rocksdb::DBVector> {
        self.db.get(key).unwrap()
    }

    // Use generators ???
    pub fn scan(&self, prefix: &[u8]) -> Vec<Row> {
        let mut rows = Vec::new();
        let mut iter = self.db.raw_iterator();
        let prefix_len = prefix.len();
        iter.seek(prefix);
        while iter.valid() {
            let key = &iter.key().unwrap();
            if &key[..prefix_len] != prefix {
                break;
            }
            rows.push(Row {
                key: key.to_vec(),
                value: iter.value().unwrap().to_vec(),
            });
            iter.next();
        }
        rows
    }
}