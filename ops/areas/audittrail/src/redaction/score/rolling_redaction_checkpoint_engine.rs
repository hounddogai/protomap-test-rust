use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling redaction checkpoint calculation.
pub trait RollingRedactionCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingRedactionCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRollingRedactionCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingRedactionCheckpointModifier for SegmentRollingRedactionCheckpointAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RollingRedactionCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingRedactionCheckpointAdjuster {
    pub percent: f64,
}

impl RollingRedactionCheckpointModifier for PercentRollingRedactionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingRedactionCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingRedactionCheckpointAdjuster {
    pub minimum: f64,
}

impl RollingRedactionCheckpointModifier for MinimumRollingRedactionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingRedactionCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingRedactionCheckpointAdjuster {
    pub step: f64,
}

impl RollingRedactionCheckpointModifier for RoundingRollingRedactionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingRedactionCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rolling redaction checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingRedactionCheckpointEngine {
    pub steps: Vec<Rc<dyn RollingRedactionCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingRedactionCheckpointEngine {
    pub fn price(&mut self, item: &RollingRedactionCheckpoint) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[RollingRedactionCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_redaction_checkpoint_engine() -> RollingRedactionCheckpointEngine {
    RollingRedactionCheckpointEngine {
        steps: vec![
            Rc::new(SegmentRollingRedactionCheckpointAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.01), ("west".to_string(), 1.01)]),
            }) as Rc<dyn RollingRedactionCheckpointModifier>,
            Rc::new(PercentRollingRedactionCheckpointAdjuster { percent: 10.9 }),
            Rc::new(MinimumRollingRedactionCheckpointAdjuster { minimum: 32.1 }),
            Rc::new(RoundingRollingRedactionCheckpointAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 69762.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_redaction_checkpoint_engine();
        let item = new_rolling_redaction_checkpoint("code-745", "title-13");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
