use std::ops::Sub;

use crate::period::Period;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
/// A receipt which will be stored to validate the payout of a node. This will then be hashed to
/// create the payment memo.
///
/// Note that this only makes sense for valid mints, hence there is no error field here.
pub struct MintingReceipt {
    pub period: Period,
    pub node_id: u32,
    pub twin_id: u32,
    pub farm_id: u32,
    pub farm_name: String,
    pub stellar_payout_address: String,
    pub measured_uptime: u64,
    /// TFT price on connection in milli USD.
    pub tft_connection_price: u64,
    pub cloud_units: CloudUnits,
    pub resource_units: ResourceUnits,
    pub resource_utilization: ResourceUtilization,
    pub reward: Reward,
    pub carbon_offset: Reward,
    /// Certification type of the node, "Certified" or "DIY".
    pub node_type: String,
    #[serde(default = "default_farming_policy_id")]
    pub farming_policy_id: u32,
    #[serde(default)]
    pub resource_rewards: ResourceRewards,
}

/// Helper function so old minting receipts which did not have a farming policy id can be
/// deserialized.
const fn default_farming_policy_id() -> u32 {
    1
}

#[derive(Serialize, Deserialize)]
pub struct ResourceRewards {
    pub cu: u64,
    pub su: u64,
    pub nu: u64,
    pub ipv4: u64,
}

/// These are the values of the initial farming policy.
impl Default for ResourceRewards {
    fn default() -> Self {
        ResourceRewards {
            cu: 2400,
            su: 1000,
            nu: 30,
            ipv4: 5,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
/// Cloud units for a node.
pub struct CloudUnits {
    pub cu: f64,
    pub su: f64,
    pub nu: f64,
}

impl Sub for CloudUnits {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            cu: self.cu - rhs.cu,
            su: self.su - rhs.su,
            nu: self.nu - rhs.nu,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
/// Payout for a node.
pub struct Reward {
    /// Reward in milli USD.
    pub musd: u64,
    /// Reward in TFT units. 1 TFT -> 1e7 units.
    pub tft: u64,
}

impl Sub for Reward {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        // If we would end up with a negative, set to 0
        Self {
            musd: if self.musd >= rhs.musd {
                self.musd - rhs.musd
            } else {
                0
            },
            tft: if self.tft >= rhs.tft {
                self.tft - rhs.tft
            } else {
                0
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
/// Resource units as reported by the node.
pub struct ResourceUnits {
    pub cru: f64,
    pub mru: f64,
    pub hru: f64,
    pub sru: f64,
}

#[derive(Serialize, Deserialize)]
/// Utilization of resoures on a node as measured through capacity reports on the chain.
pub struct ResourceUtilization {
    pub cru: f64,
    pub mru: f64,
    pub hru: f64,
    pub sru: f64,
    pub ip: f64,
}
