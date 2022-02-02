use std::fmt::Display;

use crate::{
    block::{Block, Payload},
    hash::Hashable,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
pub struct Chain {
    blocks: Vec<Block>,
}

#[cfg(not(feature = "serde"))]
pub struct Chain {
    blocks: Vec<Block>,
}

impl Chain {
    pub fn add_block(&mut self, payload: Payload) -> Result<(), MiningError> {
        let block = Block::mine(self.blocks.last().ok_or(MiningError::NoPrev)?, payload);
        self.blocks.push(block);
        Ok(())
    }

    pub fn get_blocks(&self) -> Vec<Block> {
        self.blocks.clone()
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if !self
            .blocks
            .first()
            .ok_or(ValidationError::EmptyChain)?
            .eq(&Block::genesis())
        {
            return Err(ValidationError::BadGenesisBlock);
        }

        self.blocks
            .windows(2)
            .try_for_each(|blocks| Self::validate_neighbour_block(&blocks[0], &blocks[1]))?;

        Ok(())
    }

    fn validate_neighbour_block(previous: &Block, current: &Block) -> Result<(), ValidationError> {
        if previous.get_hash() != current.get_prev_hash() {
            return Err(ValidationError::InvalidPrevHash);
        }
        if current.get_hash() != current.make_hash() {
            return Err(ValidationError::InvalidHash);
        }
        Ok(())
    }

    pub fn accept(&mut self, other: Chain) -> Result<(), ValidationError> {
        if other.len() <= self.len() {
            // if same size  we are just fine keeping our copy
            return Ok(());
        }

        other.validate()?;

        // if we get here we have validated that the incoming chain is alright, so we can append what we are missing
        let mut missing_blocks: Vec<Block> = other.blocks.into_iter().skip(self.len()).collect();
        println!("appending new blocks to chain :");
        missing_blocks.iter().for_each(|block| {
            println!("{block}");
        });
        self.blocks.append(&mut missing_blocks);
        Ok(())
    }
}

impl Default for Chain {
    fn default() -> Self {
        Self {
            blocks: vec![Block::genesis()],
        }
    }
}

impl Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.blocks
            .iter()
            .try_for_each(|block| writeln!(f, "{}", block))?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MiningError {
    NoPrev,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    EmptyChain,
    BadGenesisBlock,
    InvalidHash,
    InvalidPrevHash,
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use crate::{chain::ValidationError, hash::Hash, Block, Chain, MiningError};

    #[test]
    fn test_add_block() {
        let mut chain = Chain::default();
        assert_eq!(1, chain.blocks.len());

        chain.add_block("Hello world!".to_string()).unwrap();
        assert_eq!(2, chain.blocks.len());
    }

    #[test]
    fn test_validate_empty_chain() {
        let mut chain = Chain::default();
        chain.blocks = vec![];

        assert_eq!(chain.validate(), Err(ValidationError::EmptyChain));
    }

    #[test]
    fn test_validate_bad_genesis() {
        let mut chain = Chain::default();
        if let Some(x) = chain.blocks.get_mut(0) {
            *x = Block::new(
                SystemTime::now(),
                Hash::default(),
                "tampered genesis".to_string(),
            );
        }
        assert_eq!(chain.validate(), Err(ValidationError::BadGenesisBlock));
    }

    #[test]
    fn test_validate_bad_prev_hash() -> Result<(), MiningError> {
        let mut chain = Chain::default();

        chain.add_block("second block".to_string())?;
        chain.add_block("third block".to_string())?;

        if let Some(x) = chain.blocks.get_mut(2) {
            *x = Block::new(x.get_timestamp(), Hash::default(), x.get_payload().clone());
        }

        assert_eq!(chain.validate(), Err(ValidationError::InvalidPrevHash));
        Ok(())
    }

    #[test]
    fn test_validate_bad_hash() -> Result<(), MiningError> {
        let mut chain = Chain::default();

        chain.add_block("second block".to_string())?;
        chain.add_block("third block".to_string())?;

        if let Some(x) = chain.blocks.get_mut(2) {
            x.set_payload("tampered payload".to_string());
        }

        assert_eq!(chain.validate(), Err(ValidationError::InvalidHash));
        Ok(())
    }

    #[test]
    fn test_validate_valid_chain() -> Result<(), MiningError> {
        let mut chain = Chain::default();

        chain.add_block("second block".to_string())?;
        chain.add_block("third block".to_string())?;

        assert_eq!(chain.validate(), Ok(()));
        Ok(())
    }

    #[test]
    fn test_accept_valid_chain() -> Result<(), MiningError> {
        let mut main_chain = Chain::default();
        main_chain.add_block("second block".to_string())?;
        main_chain.add_block("third block".to_string())?;

        let mut incoming_chain = Chain::default();
        incoming_chain.add_block("second block".to_string())?;
        incoming_chain.add_block("third block".to_string())?;
        incoming_chain.add_block("fourth block".to_string())?;

        assert_eq!(3, main_chain.len());
        assert_eq!(Ok(()), main_chain.accept(incoming_chain));
        assert_eq!(4, main_chain.len());
        assert_eq!("fourth block", main_chain.blocks[3].get_payload());
        Ok(())
    }

    #[test]
    fn test_refuse_bad_hash_chain() -> Result<(), MiningError> {
        let mut main_chain = Chain::default();
        main_chain.add_block("second block".to_string())?;
        main_chain.add_block("third block".to_string())?;

        let mut incoming_chain = Chain::default();
        incoming_chain.add_block("second block".to_string())?;
        incoming_chain.add_block("tampered block".to_string())?;
        if let Some(x) = incoming_chain.blocks.get_mut(2) {
            x.set_payload("tampered payload".to_string());
        }
        incoming_chain.add_block("fourth block".to_string())?;

        assert_eq!(3, main_chain.len());
        assert_eq!(
            Err(ValidationError::InvalidHash),
            main_chain.accept(incoming_chain)
        );
        assert_eq!(3, main_chain.len());
        Ok(())
    }

    #[test]
    fn test_refuse_shorter_chain() -> Result<(), MiningError> {
        let mut main_chain = Chain::default();
        main_chain.add_block("second block".to_string())?;
        main_chain.add_block("third block".to_string())?;

        let mut incoming_chain = Chain::default();
        incoming_chain.add_block("second block".to_string())?;

        assert_eq!(3, main_chain.len());
        assert_eq!(Ok(()), main_chain.accept(incoming_chain));
        assert_eq!(3, main_chain.len());
        Ok(())
    }
}
