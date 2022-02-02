use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type Payload = String;

use crate::hash::{Hash, Hashable};

#[cfg(feature = "serde")]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Block {
    timestamp: SystemTime,
    prev_hash: Hash,
    hash: Hash,
    payload: Payload,
}

#[cfg(not(feature = "serde"))]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block {
    timestamp: SystemTime,
    prev_hash: Hash,
    hash: Hash,
    payload: Payload,
}

impl Block {
    pub fn new(timestamp: SystemTime, prev_hash: Hash, payload: Payload) -> Self {
        let mut block = Self {
            timestamp,
            prev_hash,
            hash: Hash::default(),
            payload,
        };

        block.hash = block.make_hash();
        block
    }

    pub fn genesis() -> Self {
        Self::new(
            SystemTime::from(UNIX_EPOCH),
            Hash::from_bytes(&[1]),
            "Genesis block".to_string(),
        )
    }

    pub fn mine(prev_block: &Block, payload: Payload) -> Self {
        let now = SystemTime::now();
        Self::new(now, prev_block.hash, payload)
    }

    pub fn get_prev_hash(&self) -> Hash {
        self.prev_hash
    }

    pub fn get_timestamp(&self) -> SystemTime {
        self.timestamp
    }

    pub fn get_payload(&self) -> &Payload {
        &self.payload
    }

    pub fn get_hash(&self) -> Hash {
        self.hash
    }

    #[allow(dead_code)]
    pub(crate) fn set_payload(&mut self, payload: Payload) {
        self.payload = payload;
    }
}

impl Hashable for Block {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(
            self.timestamp
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_le_bytes(),
        );

        bytes.extend(self.prev_hash.bytes());
        bytes.extend(self.payload.as_bytes());

        bytes
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::from(UNIX_EPOCH),
            prev_hash: Hash::default(),
            hash: Hash::default(),
            payload: String::default(),
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Block -")?;
        writeln!(
            f,
            "Timestamp     : {}",
            self.timestamp
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        )?;
        writeln!(f, "Previous hash : {}", self.prev_hash)?;
        writeln!(f, "Hash          : {}", self.get_hash())?;
        writeln!(f, "Data          : {}", self.payload)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Block;
    use crate::hash::Hash;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_field() {
        let timestamp = SystemTime::now();
        let prev_hash = Hash::from_bytes("123".as_bytes());
        let payload = String::from("Hello world!");
        let block: Block = Block::new(timestamp, prev_hash, payload);

        assert_eq!(block.timestamp, timestamp);
        assert_eq!(block.prev_hash, prev_hash);
        assert_eq!(block.timestamp, timestamp);
    }

    #[test]
    fn test_hash() {
        let payload_str = "Hello world!";
        let timestamp = SystemTime::now();
        let prev_hash = Hash::from_bytes("123".as_bytes());
        let payload = String::from(payload_str);
        let block: Block = Block::new(timestamp, prev_hash, payload);

        let mut block_bytes = vec![];

        block_bytes.extend(
            timestamp
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_le_bytes(),
        );
        block_bytes.extend(prev_hash.bytes());
        block_bytes.extend(payload_str.bytes());

        let expected_hash = Hash::from_bytes(&block_bytes);

        assert_eq!(expected_hash, block.get_hash());
    }
}
