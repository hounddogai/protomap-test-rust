use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout snapshot calculation.
pub trait RolloutSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutSnapshotStep for FlaggedRolloutSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RolloutSnapshotStep for TieredRolloutSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RolloutSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRolloutSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutSnapshotStep for NameRolloutSnapshotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RolloutSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutSnapshotAdjuster {
    pub percent: f64,
}

impl RolloutSnapshotStep for PercentRolloutSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollout snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutSnapshotEngine {
    pub steps: Vec<Rc<dyn RolloutSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutSnapshotEngine {
    pub fn evaluate(&mut self, item: &RolloutSnapshot) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RolloutSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollout_snapshot_engine(flags: &flags::Registry) -> RolloutSnapshotEngine {
    RolloutSnapshotEngine {
        steps: vec![
            Rc::new(FlaggedRolloutSnapshotAdjuster {
                flags: flags.clone(),
                flag: "rollout-snapshot-strict".to_string(),
                percent: -6.4,
            }) as Rc<dyn RolloutSnapshotStep>,
            Rc::new(TieredRolloutSnapshotAdjuster {
                bounds: vec![2665.0, 4650.0, 7335.0],
                rates: vec![0.84, 0.82, 1.2],
            }),
            Rc::new(NameRolloutSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.24),
                    ("west".to_string(), 0.88),
                    ("retail".to_string(), 1.08),
                ]),
            }),
            Rc::new(PercentRolloutSnapshotAdjuster { percent: -20.2 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 33863.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_snapshot_engine(&flags::new_registry());
        let item = new_rollout_snapshot("code-15", "name-486");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
