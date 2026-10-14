use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the adaptive rollout snapshot calculation.
pub trait AdaptiveRolloutSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRolloutSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveRolloutSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveRolloutSnapshotAdjuster for FlaggedAdaptiveRolloutSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveRolloutSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveRolloutSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveRolloutSnapshotAdjuster for TieredAdaptiveRolloutSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveRolloutSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveRolloutSnapshotAdjuster {
    pub cap: f64,
}

impl AdaptiveRolloutSnapshotAdjuster for CapAdaptiveRolloutSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveRolloutSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleAdaptiveRolloutSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveRolloutSnapshotAdjuster for TitleAdaptiveRolloutSnapshotAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &AdaptiveRolloutSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveRolloutSnapshotAdjuster {
    pub percent: f64,
}

impl AdaptiveRolloutSnapshotAdjuster for PercentAdaptiveRolloutSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveRolloutSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive rollout snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRolloutSnapshotEngine {
    pub steps: Vec<Rc<dyn AdaptiveRolloutSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRolloutSnapshotEngine {
    pub fn evaluate(&mut self, item: &AdaptiveRolloutSnapshot) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveRolloutSnapshotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRolloutSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_rollout_snapshot_engine(flags: &flags::Registry) -> AdaptiveRolloutSnapshotEngine {
    AdaptiveRolloutSnapshotEngine {
        steps: vec![
            Rc::new(FlaggedAdaptiveRolloutSnapshotAdjuster {
                flags: flags.clone(),
                flag: "adaptive-rollout-snapshot-primary".to_string(),
                percent: -0.0,
            }) as Rc<dyn AdaptiveRolloutSnapshotAdjuster>,
            Rc::new(TieredAdaptiveRolloutSnapshotAdjuster {
                bounds: vec![3611.0, 5266.0, 5389.0],
                rates: vec![0.96, 0.96, 1.07],
            }),
            Rc::new(CapAdaptiveRolloutSnapshotAdjuster { cap: 43850.0 }),
            Rc::new(TitleAdaptiveRolloutSnapshotAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.84)]),
            }),
            Rc::new(PercentAdaptiveRolloutSnapshotAdjuster { percent: 4.9 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 26490.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_rollout_snapshot_engine(&flags::new_registry());
        let item = new_adaptive_rollout_snapshot("code-736", "title-556");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
