use std::{collections::BTreeMap, fs};

use receipt::{MintingReceipt, ResourceRewards};

use crate::period::STANDARD_PERIOD_DURATION;

mod period;
mod receipt;

/// Directory names in the receipt directory to scan.
const DIR_NAMES: [&str; 6] = ["52", "53", "54", "55", "56", "57"];
/// Precision of 1 TFT.
const TFT_PRECISION: u64 = 10_000_000;
/// node_type value for certified nodes.
const CERTIFIED_NODE_TYPE: &str = "CERTIFIED";
/// Additional scale for percentages.
const PERCENTAGE_PRECISION: u32 = 1_000;

/// Aggregated results of a node
#[derive(Debug, Default)]
struct NodeResult {
    p52: NodePeriodResult,
    p53: NodePeriodResult,
    p54: NodePeriodResult,
    p55: NodePeriodResult,
    p56: NodePeriodResult,
    p57: NodePeriodResult,
}

impl NodeResult {
    fn is_titan(&self) -> bool {
        self.p52.is_titan()
            || self.p53.is_titan()
            || self.p54.is_titan()
            || self.p55.is_titan()
            || self.p56.is_titan()
            || self.p57.is_titan()
    }
}

impl<'a> IntoIterator for &'a NodeResult {
    type IntoIter = std::array::IntoIter<&'a NodePeriodResult, 6>;
    type Item = &'a NodePeriodResult;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter([
            &self.p52, &self.p53, &self.p54, &self.p55, &self.p56, &self.p57,
        ])
    }
}

#[derive(Debug, Default)]
struct NodePeriodResult {
    farming_policy: u32,
    uptime_percentage: u32,
    expected_payout: u64,
    actual_payout: u64,
    is_certified: bool,
}

impl NodePeriodResult {
    fn is_titan(&self) -> bool {
        self.farming_policy == 2 || (self.farming_policy == 1 && self.is_certified)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut node_receipts = BTreeMap::<_, Vec<(_, _)>>::new();
    // aggregate all the receipts
    for dir_name in DIR_NAMES {
        let period = u32::from_str_radix(dir_name, 10).expect("Dir name is period offset");
        for entry in fs::read_dir(dir_name)? {
            let entry = entry?;
            let receipt =
                serde_json::from_reader::<_, MintingReceipt>(fs::File::open(entry.path())?)?;
            node_receipts
                .entry(receipt.node_id)
                .or_default()
                .push((period, receipt));
        }
    }

    let mut node_results = BTreeMap::new();
    for (node_id, receipts) in node_receipts {
        // Technically we could allocate this map outside of the loop an reuse it everytime, but
        // this offers an implicit sanity check.
        let mut receipts_parsed = BTreeMap::new();
        for (period, receipt) in receipts {
            receipts_parsed.insert(
                period,
                NodePeriodResult {
                    farming_policy: receipt.farming_policy_id,
                    uptime_percentage: u32::min(
                        (receipt.measured_uptime * 100 * PERCENTAGE_PRECISION as u64
                            / STANDARD_PERIOD_DURATION) as u32,
                        100 * PERCENTAGE_PRECISION,
                    ),
                    expected_payout: calculate_expected_titan_reward(&receipt),
                    actual_payout: receipt.reward.tft,
                    is_certified: receipt.node_type == CERTIFIED_NODE_TYPE,
                },
            );
        }
        node_results.insert(
            node_id,
            NodeResult {
                p52: receipts_parsed.remove(&52).unwrap_or_default(),
                p53: receipts_parsed.remove(&53).unwrap_or_default(),
                p54: receipts_parsed.remove(&54).unwrap_or_default(),
                p55: receipts_parsed.remove(&55).unwrap_or_default(),
                p56: receipts_parsed.remove(&56).unwrap_or_default(),
                p57: receipts_parsed.remove(&57).unwrap_or_default(),
            },
        );
    }

    println!("node_id,p52 titan,p52 uptime,p52 expected TFT,p52 received TFT,p53 titan,p53 uptime,p53 expected TFT,p53 received TFT,p54 titan,p54 uptime,p54 expected TFT,p54 received TFT,p55 titan,p55 uptime,p55 expected TFT,p55 received TFT,p56 titan,p56 uptime,p56 expected TFT,p56 received TFT,p57 titan,p57 uptime,p57 expected TFT,p57 received TFT,Total expected TFT, Total received TFT,Difference (to send)");
    for (node_id, result) in node_results {
        // We only really care about nodes which have been a titan at some point
        if !result.is_titan() {
            continue;
        }

        let total_expected: u64 = result.into_iter().map(|r| r.expected_payout).sum();
        let total_received: u64 = result.into_iter().map(|r| r.actual_payout).sum();
        let difference = total_expected as i64 - total_received as i64;
        println!("{node_id},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            result.p52.is_titan(), format_percentage(result.p52.uptime_percentage), format_tft(result.p52.expected_payout),format_tft(result.p52.actual_payout),
            result.p53.is_titan(), format_percentage(result.p53.uptime_percentage), format_tft(result.p53.expected_payout),format_tft(result.p53.actual_payout),
            result.p54.is_titan(), format_percentage(result.p54.uptime_percentage), format_tft(result.p54.expected_payout),format_tft(result.p54.actual_payout),
            result.p55.is_titan(), format_percentage(result.p55.uptime_percentage), format_tft(result.p55.expected_payout),format_tft(result.p55.actual_payout),
            result.p56.is_titan(), format_percentage(result.p56.uptime_percentage), format_tft(result.p56.expected_payout),format_tft(result.p56.actual_payout),
            result.p57.is_titan(), format_percentage(result.p57.uptime_percentage), format_tft(result.p57.expected_payout),format_tft(result.p57.actual_payout),
            format_tft(total_expected), format_tft(total_received), format_diff_tft(difference)
        );
    }

    Ok(())
}

/// Parses an amount of TFT to it's string form.
fn format_tft(amount: u64) -> String {
    format!("{}.{:07}", amount / TFT_PRECISION, amount % TFT_PRECISION)
}

/// Parses an amount of TFT to it's string form, where the amount can potentially be negative.
fn format_diff_tft(amount: i64) -> String {
    format!(
        "{}.{:07}",
        amount / TFT_PRECISION as i64,
        amount.abs() % TFT_PRECISION as i64
    )
}

/// Farming policy 2, taken from chain.
const TITAN_RESOURCE_REWARDS: ResourceRewards = ResourceRewards {
    cu: 3000,
    su: 1250,
    nu: 38,
    ipv4: 6,
};

/// Calculate the expected reward as if the node had farming policy 2
fn calculate_expected_titan_reward(receipt: &MintingReceipt) -> u64 {
    let full_musd_reward_upscaled = ((receipt.cloud_units.cu * TFT_PRECISION as f64) as u64
        * TITAN_RESOURCE_REWARDS.cu)
        + ((receipt.cloud_units.su * TFT_PRECISION as f64) as u64 * TITAN_RESOURCE_REWARDS.su)
        + ((receipt.cloud_units.nu * TFT_PRECISION as f64) as u64 * TITAN_RESOURCE_REWARDS.nu)
        + ((receipt.resource_utilization.ip * TFT_PRECISION as f64) as u64
            * TITAN_RESOURCE_REWARDS.ipv4);

    // Don't divide by TFT_PRECISION as the conenction price is expressed as mUSD/TFT which is
    // actually mUSD / TFT_PRECISION
    let full_tft_reward = full_musd_reward_upscaled / receipt.tft_connection_price;

    // scale, use default period duration so we account for nodes which did not come online until
    // the period already started
    full_tft_reward * receipt.measured_uptime / STANDARD_PERIOD_DURATION
}

/// Format a percentage with 3 digits of precision
fn format_percentage(p: u32) -> String {
    format!("{}.{}%", p / PERCENTAGE_PRECISION, p % PERCENTAGE_PRECISION)
}
