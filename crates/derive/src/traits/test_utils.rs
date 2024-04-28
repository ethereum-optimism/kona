//! Test Utilities for derive traits

use crate::{
    traits::{AsyncIterator, ChainProvider, DataAvailabilityProvider, L2ChainProvider},
    types::{FinalizedHeadSignal, PlasmaError, StageError, StageResult},
};
use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
use alloy_consensus::{Header, Receipt, TxEnvelope};
use alloy_primitives::{Address, Bytes, B256};
use anyhow::Result;
use async_trait::async_trait;
use core::fmt::Debug;
use hashbrown::HashMap;
use kona_primitives::{
    BlockID, BlockInfo, L2BlockInfo, L2ExecutionPayloadEnvelope, RollupConfig, SystemConfig,
};

use super::PlasmaInputFetcher;

/// Mock data iterator
#[derive(Debug, Default, PartialEq)]
pub struct TestIter {
    /// Holds open data calls with args for assertions.
    pub(crate) open_data_calls: Vec<(BlockInfo, Address)>,
    /// A queue of results to return as the next iterated data.
    pub(crate) results: Vec<StageResult<Bytes>>,
}

#[async_trait]
impl AsyncIterator for TestIter {
    type Item = Bytes;

    async fn next(&mut self) -> Option<StageResult<Self::Item>> {
        Some(self.results.pop().unwrap_or_else(|| Err(StageError::Eof)))
    }
}

/// Mock data availability provider
#[derive(Debug, Default)]
pub struct TestDAP {
    /// Specifies the stage results the test iter returns as data.
    pub(crate) results: Vec<StageResult<Bytes>>,
}

#[async_trait]
impl DataAvailabilityProvider for TestDAP {
    type Item = Bytes;
    type DataIter = TestIter;

    async fn open_data(
        &self,
        block_ref: &BlockInfo,
        batcher_address: Address,
    ) -> Result<Self::DataIter> {
        // Construct a new vec of results to return.
        let results = self
            .results
            .iter()
            .map(|i| match i {
                Ok(r) => Ok(r.clone()),
                Err(_) => Err(StageError::Eof),
            })
            .collect::<Vec<StageResult<Bytes>>>();
        Ok(TestIter { open_data_calls: vec![(*block_ref, batcher_address)], results })
    }
}

/// A mock chain provider for testing.
#[derive(Debug, Clone, Default)]
pub struct TestChainProvider {
    /// Maps block numbers to block information using a tuple list.
    pub blocks: Vec<(u64, BlockInfo)>,
    /// Maps block hashes to header information using a tuple list.
    pub headers: Vec<(B256, Header)>,
    /// Maps block hashes to receipts using a tuple list.
    pub receipts: Vec<(B256, Vec<Receipt>)>,
}

impl TestChainProvider {
    /// Insert a block into the mock chain provider.
    pub fn insert_block(&mut self, number: u64, block: BlockInfo) {
        self.blocks.push((number, block));
    }

    /// Insert receipts into the mock chain provider.
    pub fn insert_receipts(&mut self, hash: B256, receipts: Vec<Receipt>) {
        self.receipts.push((hash, receipts));
    }

    /// Insert a header into the mock chain provider.
    pub fn insert_header(&mut self, hash: B256, header: Header) {
        self.headers.push((hash, header));
    }

    /// Clears headers from the mock chain provider.
    pub fn clear_headers(&mut self) {
        self.headers.clear();
    }

    /// Clears blocks from the mock chain provider.
    pub fn clear_blocks(&mut self) {
        self.blocks.clear();
    }

    /// Clears receipts from the mock chain provider.
    pub fn clear_receipts(&mut self) {
        self.receipts.clear();
    }

    /// Clears all blocks and receipts from the mock chain provider.
    pub fn clear(&mut self) {
        self.clear_blocks();
        self.clear_receipts();
        self.clear_headers();
    }
}

#[async_trait]
impl ChainProvider for TestChainProvider {
    async fn header_by_hash(&mut self, hash: B256) -> Result<Header> {
        if let Some((_, header)) = self.headers.iter().find(|(_, b)| b.hash_slow() == hash) {
            Ok(header.clone())
        } else {
            Err(anyhow::anyhow!("Header not found"))
        }
    }

    async fn block_info_by_number(&mut self, _number: u64) -> Result<BlockInfo> {
        if let Some((_, block)) = self.blocks.iter().find(|(n, _)| *n == _number) {
            Ok(*block)
        } else {
            Err(anyhow::anyhow!("Block not found"))
        }
    }

    async fn receipts_by_hash(&mut self, _hash: B256) -> Result<Vec<Receipt>> {
        if let Some((_, receipts)) = self.receipts.iter().find(|(h, _)| *h == _hash) {
            Ok(receipts.clone())
        } else {
            Err(anyhow::anyhow!("Receipts not found"))
        }
    }

    async fn block_info_and_transactions_by_hash(
        &mut self,
        hash: B256,
    ) -> Result<(BlockInfo, Vec<TxEnvelope>)> {
        let block = self
            .blocks
            .iter()
            .find(|(_, b)| b.hash == hash)
            .map(|(_, b)| *b)
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        Ok((block, Vec::new()))
    }
}

/// An [L2ChainProvider] implementation for testing.
#[derive(Debug, Default)]
pub struct TestL2ChainProvider {
    /// Blocks
    pub blocks: Vec<L2BlockInfo>,
    /// Short circuit the block return to be the first block.
    pub short_circuit: bool,
    /// Payloads
    pub payloads: Vec<L2ExecutionPayloadEnvelope>,
    /// System configs
    pub system_configs: HashMap<u64, SystemConfig>,
}

impl TestL2ChainProvider {
    /// Creates a new [MockBlockFetcher] with the given origin and batches.
    pub fn new(
        blocks: Vec<L2BlockInfo>,
        payloads: Vec<L2ExecutionPayloadEnvelope>,
        system_configs: HashMap<u64, SystemConfig>,
    ) -> Self {
        Self { blocks, short_circuit: false, payloads, system_configs }
    }
}

#[async_trait]
impl L2ChainProvider for TestL2ChainProvider {
    async fn l2_block_info_by_number(&mut self, number: u64) -> Result<L2BlockInfo> {
        if self.short_circuit {
            return self.blocks.first().copied().ok_or_else(|| anyhow::anyhow!("Block not found"));
        }
        self.blocks
            .iter()
            .find(|b| b.block_info.number == number)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Block not found"))
    }

    async fn payload_by_number(&mut self, number: u64) -> Result<L2ExecutionPayloadEnvelope> {
        self.payloads
            .iter()
            .find(|p| p.execution_payload.block_number == number)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Payload not found"))
    }

    async fn system_config_by_number(
        &mut self,
        number: u64,
        _: Arc<RollupConfig>,
    ) -> Result<SystemConfig> {
        self.system_configs
            .get(&number)
            .ok_or_else(|| anyhow::anyhow!("System config not found"))
            .cloned()
    }
}

/// A mock plasma input fetcher for testing.
#[derive(Debug, Clone, Default)]
pub struct TestPlasmaInputFetcher {
    /// Inputs to return.
    pub inputs: Vec<Result<Bytes, PlasmaError>>,
    /// Advance L1 origin results.
    pub advances: Vec<Result<(), PlasmaError>>,
    /// Reset results.
    pub resets: Vec<Result<(), PlasmaError>>,
}

#[async_trait]
impl PlasmaInputFetcher<TestChainProvider> for TestPlasmaInputFetcher {
    async fn get_input(
        &mut self,
        _fetcher: &TestChainProvider,
        _commitment: Bytes,
        _block: BlockID,
    ) -> Option<Result<Bytes, PlasmaError>> {
        self.inputs.pop()
    }

    async fn advance_l1_origin(
        &mut self,
        _fetcher: &TestChainProvider,
        _block: BlockID,
    ) -> Option<Result<(), PlasmaError>> {
        self.advances.pop()
    }

    async fn reset(
        &mut self,
        _block_number: BlockInfo,
        _cfg: SystemConfig,
    ) -> Option<Result<(), PlasmaError>> {
        self.resets.pop()
    }

    async fn finalize(&mut self, _block_number: BlockInfo) -> Option<Result<(), PlasmaError>> {
        None
    }

    fn on_finalized_head_signal(&mut self, _block_number: FinalizedHeadSignal) {}
}
