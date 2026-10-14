use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling runbook override calculation.
pub trait RollingRunbookOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingRunbookOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingRunbookOverrideAdjuster {
    pub percent: f64,
}

impl RollingRunbookOverrideModifier for PercentRollingRunbookOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingRunbookOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingRunbookOverrideAdjuster {
    pub cap: f64,
}

impl RollingRunbookOverrideModifier for CapRollingRunbookOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingRunbookOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRollingRunbookOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingRunbookOverrideModifier for CategoryRollingRunbookOverrideAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RollingRunbookOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingRunbookOverrideAdjuster {
    pub step: f64,
}

impl RollingRunbookOverrideModifier for RoundingRollingRunbookOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingRunbookOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookOverrideTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl RollingRunbookOverrideModifier for RollingRunbookOverrideTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &RollingRunbookOverride, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

/// Runs the rolling runbook override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingRunbookOverrideEngine {
    pub steps: Vec<Rc<dyn RollingRunbookOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingRunbookOverrideEngine {
    pub fn price(&mut self, item: &RollingRunbookOverride) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingRunbookOverrideModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingRunbookOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_runbook_override_engine() -> RollingRunbookOverrideEngine {
    RollingRunbookOverrideEngine {
        steps: vec![
            Rc::new(PercentRollingRunbookOverrideAdjuster { percent: -2.8 }) as Rc<dyn RollingRunbookOverrideModifier>,
            Rc::new(CapRollingRunbookOverrideAdjuster { cap: 41176.0 }),
            Rc::new(CategoryRollingRunbookOverrideAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.15), ("east".to_string(), 1.24)]),
            }),
            Rc::new(RoundingRollingRunbookOverrideAdjuster { step: 0.25 }),
            Rc::new(RollingRunbookOverrideTtlSecondsOffsetAdjuster { weight: 0.2 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 90623.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_runbook_override_engine();
        let item = new_rolling_runbook_override("code-681", "label-259");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
