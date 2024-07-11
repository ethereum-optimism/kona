//! Contains the L2-specifc contstructs of the client program.

mod trie_hinter;
pub use trie_hinter::TrieDBHintWriter;

mod chain_provider;
pub use chain_provider::OracleL2ChainProvider;

#[cfg(not(feature = "no-io"))]
mod precompiles;
#[cfg(not(feature = "no-io"))]
pub use precompiles::FPVMPrecompileOverride;
