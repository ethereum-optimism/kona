//! A mock implementation of the [`BatchStream`] stage for testing.

use crate::{
    batch::Batch,
    errors::{PipelineError, PipelineResult},
    stages::batch_stream::BatchStreamProvider,
    traits::{OriginAdvancer, OriginProvider, ResettableStage},
};
use alloc::{boxed::Box, vec::Vec};
use async_trait::async_trait;
use op_alloy_genesis::SystemConfig;
use op_alloy_protocol::BlockInfo;

/// A mock provider for the `BatchStream` stage.
#[derive(Debug, Default)]
pub struct MockBatchStreamProvider {
    /// The origin of the L1 block.
    pub origin: Option<BlockInfo>,
    /// A list of batches to return.
    pub batches: Vec<PipelineResult<Batch>>,
}

impl MockBatchStreamProvider {
    /// Creates a new [MockBatchStreamProvider] with the given origin and batches.
    pub fn new(batches: Vec<PipelineResult<Batch>>) -> Self {
        Self { origin: Some(BlockInfo::default()), batches }
    }
}

impl OriginProvider for MockBatchStreamProvider {
    fn origin(&self) -> Option<BlockInfo> {
        self.origin
    }
}

#[async_trait]
impl BatchStreamProvider for MockBatchStreamProvider {
    fn flush(&mut self) {}

    async fn next_batch(&mut self) -> PipelineResult<Batch> {
        self.batches.pop().ok_or(PipelineError::Eof.temp())?
    }
}

#[async_trait]
impl OriginAdvancer for MockBatchStreamProvider {
    async fn advance_origin(&mut self) -> PipelineResult<()> {
        Ok(())
    }
}

#[async_trait]
impl ResettableStage for MockBatchStreamProvider {
    async fn reset(&mut self, _base: BlockInfo, _cfg: &SystemConfig) -> PipelineResult<()> {
        Ok(())
    }
}
