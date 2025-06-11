use zcash_primitives::consensus::{BlockHeight, BranchId, NetworkUpgrade, Parameters};
use crate::BitcoinZMainNetwork;

// BitcoinZ-specific branch IDs
// These need to be confirmed with BitcoinZ developers
pub const BITCOINZ_OVERWINTER_BRANCH_ID: u32 = 0x5ba8_1b19; // Same as Zcash for now
pub const BITCOINZ_SAPLING_BRANCH_ID: u32 = 0x76b8_09bb;    // Same as Zcash for now
pub const BITCOINZ_BLOSSOM_BRANCH_ID: u32 = 0x2bb4_0e60;    // Same as Zcash for now
pub const BITCOINZ_HEARTWOOD_BRANCH_ID: u32 = 0xf5b9_230b;  // Same as Zcash for now
pub const BITCOINZ_CANOPY_BRANCH_ID: u32 = 0xe9ff_75a6;     // Same as Zcash for now

/// Get the BitcoinZ branch ID for a given height
/// This function maps BitcoinZ network upgrade heights to their corresponding branch IDs
pub fn bitcoinz_branch_id_for_height(network: &BitcoinZMainNetwork, height: BlockHeight) -> BranchId {
    // Check which network upgrade is active at this height
    if let Some(canopy_height) = network.activation_height(NetworkUpgrade::Canopy) {
        if height >= canopy_height {
            return BranchId::Canopy;
        }
    }
    
    if let Some(heartwood_height) = network.activation_height(NetworkUpgrade::Heartwood) {
        if height >= heartwood_height {
            return BranchId::Heartwood;
        }
    }
    
    if let Some(blossom_height) = network.activation_height(NetworkUpgrade::Blossom) {
        if height >= blossom_height {
            return BranchId::Blossom;
        }
    }
    
    if let Some(sapling_height) = network.activation_height(NetworkUpgrade::Sapling) {
        if height >= sapling_height {
            return BranchId::Sapling;
        }
    }
    
    if let Some(overwinter_height) = network.activation_height(NetworkUpgrade::Overwinter) {
        if height >= overwinter_height {
            return BranchId::Overwinter;
        }
    }
    
    // Pre-Overwinter
    BranchId::Sprout
}

/// Convert a BitcoinZ branch ID value to BranchId enum
/// This is needed because BitcoinZ might use different numeric values than Zcash
pub fn bitcoinz_branch_id_from_u32(value: u32) -> Result<BranchId, &'static str> {
    match value {
        0 => Ok(BranchId::Sprout),
        BITCOINZ_OVERWINTER_BRANCH_ID => Ok(BranchId::Overwinter),
        BITCOINZ_SAPLING_BRANCH_ID => Ok(BranchId::Sapling),
        BITCOINZ_BLOSSOM_BRANCH_ID => Ok(BranchId::Blossom),
        BITCOINZ_HEARTWOOD_BRANCH_ID => Ok(BranchId::Heartwood),
        BITCOINZ_CANOPY_BRANCH_ID => Ok(BranchId::Canopy),
        _ => Err("Unknown BitcoinZ consensus branch ID"),
    }
}

/// Get the numeric branch ID value for BitcoinZ
pub fn bitcoinz_branch_id_to_u32(branch_id: BranchId) -> u32 {
    match branch_id {
        BranchId::Sprout => 0,
        BranchId::Overwinter => BITCOINZ_OVERWINTER_BRANCH_ID,
        BranchId::Sapling => BITCOINZ_SAPLING_BRANCH_ID,
        BranchId::Blossom => BITCOINZ_BLOSSOM_BRANCH_ID,
        BranchId::Heartwood => BITCOINZ_HEARTWOOD_BRANCH_ID,
        BranchId::Canopy => BITCOINZ_CANOPY_BRANCH_ID,
        BranchId::Nu5 => panic!("BitcoinZ does not support Nu5"),
        #[cfg(feature = "zfuture")]
        BranchId::ZFuture => panic!("BitcoinZ does not support ZFuture"),
    }
}