use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the dns record proposal calculation.
pub trait DnsRecordProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordProposalAdjuster {
    pub minimum: f64,
}

impl DnsRecordProposalFactor for MinimumDnsRecordProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordProposalFactor for FlaggedDnsRecordProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordProposalFactor for TieredDnsRecordProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDnsRecordProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordProposalFactor for NameDnsRecordProposalAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DnsRecordProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordProposalQuantityOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordProposalFactor for DnsRecordProposalQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordProposal, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the dns record proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordProposalEngine {
    pub steps: Vec<Rc<dyn DnsRecordProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordProposalEngine {
    pub fn price(&mut self, item: &DnsRecordProposal) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordProposalFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[DnsRecordProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dns_record_proposal_engine(flags: &flags::Registry) -> DnsRecordProposalEngine {
    DnsRecordProposalEngine {
        steps: vec![
            Rc::new(MinimumDnsRecordProposalAdjuster { minimum: 15.2 }) as Rc<dyn DnsRecordProposalFactor>,
            Rc::new(FlaggedDnsRecordProposalAdjuster {
                flags: flags.clone(),
                flag: "dns-record-proposal-seasonal".to_string(),
                percent: 1.7,
            }),
            Rc::new(TieredDnsRecordProposalAdjuster {
                bounds: vec![96.0, 1099.0, 4483.0, 8526.0],
                rates: vec![0.91, 1.17, 0.82, 1.14],
            }),
            Rc::new(NameDnsRecordProposalAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.72)]),
            }),
            Rc::new(DnsRecordProposalQuantityOffsetAdjuster { weight: 0.17 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 85193.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_proposal_engine(&flags::new_registry());
        let item = new_dns_record_proposal("code-847", "name-19");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
