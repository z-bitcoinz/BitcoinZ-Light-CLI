use zcash_primitives::{
    consensus::{self, BlockHeight, Parameters},
    transaction::{
        self,
        components::{Amount, TxOut, transparent},
        Transaction, TxVersion,
        builder::Builder,
    },
};
use zcash_primitives::sapling::prover::TxProver;
use orchard::Anchor;
use crate::BitcoinZMainNetwork;

/// Create a Builder for BitcoinZ transactions
/// This attempts to create transactions that are compatible with BitcoinZ's validation
pub fn create_bitcoinz_builder(
    params: BitcoinZMainNetwork,
    height: BlockHeight,
    anchor: Anchor,
) -> Builder<'static, BitcoinZMainNetwork, rand::rngs::OsRng> {
    // Use the standard builder but with BitcoinZ parameters
    Builder::new_with_orchard(params, height, anchor)
}

/// Check if we should use a simplified transaction format for BitcoinZ
pub fn should_use_simple_tx(
    transparent_inputs: usize,
    transparent_outputs: usize,
    shielded_inputs: usize,
    shielded_outputs: usize,
) -> bool {
    // For transparent-only transactions, we might want to use a simpler format
    shielded_inputs == 0 && shielded_outputs == 0
}

/// Attempt to create a transaction without Sapling components for BitcoinZ
/// This is a workaround for the binding signature issue
pub fn build_transparent_only_tx<P: TxProver>(
    builder: Builder<'static, BitcoinZMainNetwork, rand::rngs::OsRng>,
    prover: &P,
) -> Result<(Transaction, transaction::builder::SaplingMetadata), transaction::builder::Error> {
    // Build the transaction normally
    // The builder should automatically handle transparent-only transactions
    // without adding Sapling components if there are no shielded inputs/outputs
    builder.build(prover)
}