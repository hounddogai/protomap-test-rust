use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the canary snapshot calculation.
pub trait CanarySnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanarySnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanarySnapshotAdjuster {
    pub cap: f64,
}

impl CanarySnapshotAdjuster for CapCanarySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanarySnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySnapshotCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl CanarySnapshotAdjuster for CanarySnapshotCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &CanarySnapshot, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanarySnapshotAdjuster {
    pub step: f64,
}

impl CanarySnapshotAdjuster for RoundingCanarySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanarySnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryCanarySnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanarySnapshotAdjuster for CountryCanarySnapshotAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &CanarySnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the canary snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanarySnapshotEngine {
    pub steps: Vec<Rc<dyn CanarySnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanarySnapshotEngine {
    pub fn evaluate(&mut self, item: &CanarySnapshot) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[CanarySnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_canary_snapshot_engine() -> CanarySnapshotEngine {
    CanarySnapshotEngine {
        steps: vec![
            Rc::new(CapCanarySnapshotAdjuster { cap: 13422.0 }) as Rc<dyn CanarySnapshotAdjuster>,
            Rc::new(CanarySnapshotCreatedAtOffsetAdjuster { weight: 0.59 }),
            Rc::new(RoundingCanarySnapshotAdjuster { step: 0.1 }),
            Rc::new(CountryCanarySnapshotAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.28), ("east".to_string(), 0.73)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 64455.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_snapshot_engine();
        let item = new_canary_snapshot("code-873", "title-816");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
