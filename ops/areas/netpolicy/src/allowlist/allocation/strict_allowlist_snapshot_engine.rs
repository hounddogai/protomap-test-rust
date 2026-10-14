use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the strict allowlist snapshot calculation.
pub trait StrictAllowlistSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictAllowlistSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictAllowlistSnapshotAdjuster {
    pub minimum: f64,
}

impl StrictAllowlistSnapshotStep for MinimumStrictAllowlistSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictAllowlistSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictAllowlistSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictAllowlistSnapshotStep for FlaggedStrictAllowlistSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictAllowlistSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelStrictAllowlistSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictAllowlistSnapshotStep for LabelStrictAllowlistSnapshotAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &StrictAllowlistSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictAllowlistSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictAllowlistSnapshotStep for TieredStrictAllowlistSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictAllowlistSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the strict allowlist snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictAllowlistSnapshotEngine {
    pub steps: Vec<Rc<dyn StrictAllowlistSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictAllowlistSnapshotEngine {
    pub fn compute(&mut self, item: &StrictAllowlistSnapshot) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictAllowlistSnapshotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictAllowlistSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_allowlist_snapshot_engine(flags: &flags::Registry) -> StrictAllowlistSnapshotEngine {
    StrictAllowlistSnapshotEngine {
        steps: vec![
            Rc::new(MinimumStrictAllowlistSnapshotAdjuster { minimum: 11.7 }) as Rc<dyn StrictAllowlistSnapshotStep>,
            Rc::new(FlaggedStrictAllowlistSnapshotAdjuster {
                flags: flags.clone(),
                flag: "strict-allowlist-snapshot-weighted".to_string(),
                percent: -3.8,
            }),
            Rc::new(LabelStrictAllowlistSnapshotAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.29)]),
            }),
            Rc::new(TieredStrictAllowlistSnapshotAdjuster {
                bounds: vec![1697.0, 5929.0],
                rates: vec![0.89, 0.81],
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 69064.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_allowlist_snapshot_engine(&flags::new_registry());
        let item = new_strict_allowlist_snapshot("code-187", "label-76");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
