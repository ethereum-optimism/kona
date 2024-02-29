//! The Span Batch Type

use crate::types::spans::{RawSpanBatch, SpanBatchBits, SpanBatchPayload, SpanBatchPrefix};
use crate::types::SpanBatchElement;
use alloc::vec;
use alloc::vec::Vec;
use alloy_primitives::{FixedBytes, U64};

/// The span batch contains the input to build a span of L2 blocks in derived form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanBatch {
    /// First 20 bytes of the first block's parent hash
    pub parent_check: FixedBytes<20>,
    /// First 20 bytes of the last block's L1 origin hash
    pub l1_origin_check: FixedBytes<20>,
    /// List of block input in derived form
    pub batches: Vec<SpanBatchElement>,
}

impl SpanBatch {
    /// Returns the timestamp for the first batch in the span.
    pub fn get_timestamp(&self) -> u64 {
        self.batches[0].timestamp
    }

    /// Converts the span batch to a raw span batch.
    pub fn to_raw_span_batch(
        &self,
        origin_changed_bit: u8,
        genesis_timestamp: u64,
        chain_id: u64,
    ) -> RawSpanBatch {
        let mut block_tx_counts = Vec::new();
        let mut txs = Vec::new();
        for batch in &self.batches {
            block_tx_counts.push(batch.transactions.len() as u64);
            for tx in &batch.transactions {
                txs.extend_from_slice(&tx.0);
            }
        }

        RawSpanBatch {
            prefix: SpanBatchPrefix {
                rel_timestamp: U64::from(self.get_timestamp() - genesis_timestamp),
                l1_origin_num: U64::from(chain_id),
                parent_check: self.parent_check,
                l1_origin_check: self.l1_origin_check,
            },
            payload: SpanBatchPayload {
                block_count: U64::from(self.batches.len() as u64),
                origin_bits: SpanBatchBits(vec![origin_changed_bit; self.batches.len()]),
                block_tx_counts,
                txs,
            },
        }
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;

    fn setup_test() -> &[u8] {
        std::include_bytes!("testdata/span_batch")
    }

    #[test]
    fn test_span_batch() {
        let data = setup_test();
        let span_batch = SpanBatch::decode(&mut data.as_ref()).unwrap();
        assert_eq!(span_batch.batches.len(), 1);
        assert_eq!(span_batch.batches[0].transactions.len(), 1);
    }
}
