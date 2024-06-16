#![doc = include_str!("../README.md")]
#![warn(missing_debug_implementations, missing_docs, unreachable_pub, rustdoc::all)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![no_std]
#![cfg_attr(any(target_arch = "mips", target_arch = "riscv64"), no_main)]

use alloc::sync::Arc;
use alloy_consensus::Header;
use kona_client::{
    l1::{DerivationDriver, OracleBlobProvider, OracleL1ChainProvider},
    l2::{OracleL2ChainProvider, StatelessL2BlockExecutor, TrieDBHintWriter},
    BootInfo, CachingOracle,
};
use kona_common_proc::client_entry;

extern crate alloc;

/// The size of the LRU cache in the oracle.
const ORACLE_LRU_SIZE: usize = 1024;

#[client_entry(0x77359400)]
fn main() -> Result<()> {
    #[cfg(feature = "tracing-subscriber")]
    init_tracing_subscriber(3)?;

    kona_common::block_on(async move {
        ////////////////////////////////////////////////////////////////
        //                          PROLOGUE                          //
        ////////////////////////////////////////////////////////////////

        let oracle = Arc::new(CachingOracle::new(ORACLE_LRU_SIZE));
        let boot = Arc::new(BootInfo::load(oracle.as_ref()).await?);
        let l1_provider = OracleL1ChainProvider::new(boot.clone(), oracle.clone());
        let l2_provider = OracleL2ChainProvider::new(boot.clone(), oracle.clone());
        let beacon = OracleBlobProvider::new(oracle.clone());

        ////////////////////////////////////////////////////////////////
        //                   DERIVATION & EXECUTION                   //
        ////////////////////////////////////////////////////////////////

        let mut driver = DerivationDriver::new(
            boot.as_ref(),
            oracle.as_ref(),
            beacon,
            l1_provider,
            l2_provider.clone(),
        )
        .await?;
        let attributes = driver.produce_disputed_payload().await?;

        let cfg = Arc::new(boot.rollup_config.clone());
        let mut executor = StatelessL2BlockExecutor::new(
            cfg,
            driver.l2_safe_head_header().clone(),
            l2_provider,
            TrieDBHintWriter,
        );
        let Header { number, .. } = *executor.execute_payload(attributes.attributes)?;
        let output_root = executor.compute_output_root()?;

        ////////////////////////////////////////////////////////////////
        //                          EPILOGUE                          //
        ////////////////////////////////////////////////////////////////

        assert_eq!(number, boot.l2_claim_block);
        assert_eq!(output_root, boot.l2_claim);

        tracing::info!(
            target: "client",
            "Successfully validated L2 block #{number} with output root {output_root}",
            number = number,
            output_root = output_root
        );

        Ok::<_, anyhow::Error>(())
    })
}

/// Initializes the tracing subscriber
///
/// # Arguments
/// * `verbosity_level` - The verbosity level (0-4)
///
/// # Returns
/// * `Result<()>` - Ok if successful, Err otherwise.
#[cfg(feature = "tracing-subscriber")]
pub fn init_tracing_subscriber(verbosity_level: u8) -> anyhow::Result<()> {
    use anyhow::anyhow;
    use tracing::Level;

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(match verbosity_level {
            0 => Level::ERROR,
            1 => Level::WARN,
            2 => Level::INFO,
            3 => Level::DEBUG,
            _ => Level::TRACE,
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber).map_err(|e| anyhow!(e))
}
