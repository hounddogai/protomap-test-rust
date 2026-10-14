use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted rule snapshot calculation.
pub trait WeightedRuleSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRuleSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedRuleSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedRuleSnapshotAdjuster for FlaggedWeightedRuleSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedRuleSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleSnapshotCeilingOffsetAdjuster {
    pub weight: f64,
}

impl WeightedRuleSnapshotAdjuster for WeightedRuleSnapshotCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedRuleSnapshot, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedRuleSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedRuleSnapshotAdjuster for TieredWeightedRuleSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedRuleSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedRuleSnapshotAdjuster {
    pub cap: f64,
}

impl WeightedRuleSnapshotAdjuster for CapWeightedRuleSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedRuleSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedRuleSnapshotAdjuster {
    pub percent: f64,
}

impl WeightedRuleSnapshotAdjuster for PercentWeightedRuleSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedRuleSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted rule snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRuleSnapshotEngine {
    pub steps: Vec<Rc<dyn WeightedRuleSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRuleSnapshotEngine {
    pub fn price(&mut self, item: &WeightedRuleSnapshot) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedRuleSnapshotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedRuleSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_rule_snapshot_engine(flags: &flags::Registry) -> WeightedRuleSnapshotEngine {
    WeightedRuleSnapshotEngine {
        steps: vec![
            Rc::new(FlaggedWeightedRuleSnapshotAdjuster {
                flags: flags.clone(),
                flag: "weighted-rule-snapshot-pending".to_string(),
                percent: 1.4,
            }) as Rc<dyn WeightedRuleSnapshotAdjuster>,
            Rc::new(WeightedRuleSnapshotCeilingOffsetAdjuster { weight: 0.16 }),
            Rc::new(TieredWeightedRuleSnapshotAdjuster {
                bounds: vec![6071.0, 6954.0, 8994.0],
                rates: vec![0.9, 1.18, 1.06],
            }),
            Rc::new(CapWeightedRuleSnapshotAdjuster { cap: 3398.0 }),
            Rc::new(PercentWeightedRuleSnapshotAdjuster { percent: 2.0 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 99525.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_rule_snapshot_engine(&flags::new_registry());
        let item = new_weighted_rule_snapshot("code-621", "title-921");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
