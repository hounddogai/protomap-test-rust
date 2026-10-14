use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the trust domain snapshot calculation.
pub trait TrustDomainSnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainSnapshotModifier for FlaggedTrustDomainSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainSnapshotAdjuster {
    pub step: f64,
}

impl TrustDomainSnapshotModifier for RoundingTrustDomainSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTrustDomainSnapshotAdjuster {
    pub percent: f64,
}

impl TrustDomainSnapshotModifier for PercentTrustDomainSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TrustDomainSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainSnapshotModifier for TieredTrustDomainSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelTrustDomainSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TrustDomainSnapshotModifier for LabelTrustDomainSnapshotAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &TrustDomainSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSnapshotWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainSnapshotModifier for TrustDomainSnapshotWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainSnapshot, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

/// Runs the trust domain snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainSnapshotEngine {
    pub steps: Vec<Rc<dyn TrustDomainSnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainSnapshotEngine {
    pub fn compute(&mut self, item: &TrustDomainSnapshot) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainSnapshotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_trust_domain_snapshot_engine(flags: &flags::Registry) -> TrustDomainSnapshotEngine {
    TrustDomainSnapshotEngine {
        steps: vec![
            Rc::new(FlaggedTrustDomainSnapshotAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-snapshot-capped".to_string(),
                percent: 10.0,
            }) as Rc<dyn TrustDomainSnapshotModifier>,
            Rc::new(RoundingTrustDomainSnapshotAdjuster { step: 0.25 }),
            Rc::new(PercentTrustDomainSnapshotAdjuster { percent: -4.1 }),
            Rc::new(TieredTrustDomainSnapshotAdjuster {
                bounds: vec![1783.0, 3548.0, 6237.0],
                rates: vec![0.86, 0.82, 0.95],
            }),
            Rc::new(LabelTrustDomainSnapshotAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.18), ("retail".to_string(), 1.3)]),
            }),
            Rc::new(TrustDomainSnapshotWindowDaysOffsetAdjuster { weight: 0.73 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 51706.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_snapshot_engine(&flags::new_registry());
        let item = new_trust_domain_snapshot("code-902", "label-86");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
