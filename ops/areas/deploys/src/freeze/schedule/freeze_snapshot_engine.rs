use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the freeze snapshot calculation.
pub trait FreezeSnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFreezeSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeSnapshotModifier for TitleFreezeSnapshotAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FreezeSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezeSnapshotAdjuster {
    pub percent: f64,
}

impl FreezeSnapshotModifier for PercentFreezeSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezeSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeSnapshotAdjuster {
    pub minimum: f64,
}

impl FreezeSnapshotModifier for MinimumFreezeSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the freeze snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeSnapshotEngine {
    pub steps: Vec<Rc<dyn FreezeSnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeSnapshotEngine {
    pub fn price(&mut self, item: &FreezeSnapshot) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FreezeSnapshotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FreezeSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_freeze_snapshot_engine() -> FreezeSnapshotEngine {
    FreezeSnapshotEngine {
        steps: vec![
            Rc::new(TitleFreezeSnapshotAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.73), ("east".to_string(), 0.81)]),
            }) as Rc<dyn FreezeSnapshotModifier>,
            Rc::new(PercentFreezeSnapshotAdjuster { percent: 14.6 }),
            Rc::new(MinimumFreezeSnapshotAdjuster { minimum: 41.7 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 72581.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_snapshot_engine();
        let item = new_freeze_snapshot("code-951", "title-640");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
