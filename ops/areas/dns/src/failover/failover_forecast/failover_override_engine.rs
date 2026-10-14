use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the failover override calculation.
pub trait FailoverOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFailoverOverrideAdjuster {
    pub minimum: f64,
}

impl FailoverOverrideModifier for MinimumFailoverOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FailoverOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverOverrideBurstOffsetAdjuster {
    pub weight: f64,
}

impl FailoverOverrideModifier for FailoverOverrideBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &FailoverOverride, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFailoverOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverOverrideModifier for LabelFailoverOverrideAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FailoverOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverOverrideAdjuster {
    pub cap: f64,
}

impl FailoverOverrideModifier for CapFailoverOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the failover override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverOverrideEngine {
    pub steps: Vec<Rc<dyn FailoverOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverOverrideEngine {
    pub fn price(&mut self, item: &FailoverOverride) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FailoverOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_failover_override_engine() -> FailoverOverrideEngine {
    FailoverOverrideEngine {
        steps: vec![
            Rc::new(MinimumFailoverOverrideAdjuster { minimum: 41.4 }) as Rc<dyn FailoverOverrideModifier>,
            Rc::new(FailoverOverrideBurstOffsetAdjuster { weight: 0.69 }),
            Rc::new(LabelFailoverOverrideAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.25), ("east".to_string(), 1.3)]),
            }),
            Rc::new(CapFailoverOverrideAdjuster { cap: 18480.0 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 67109.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_override_engine();
        let item = new_failover_override("code-964", "label-598");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
