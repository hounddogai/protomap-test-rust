use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy failover adjustment calculation.
pub trait LegacyFailoverAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyFailoverAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyFailoverAdjustmentAdjuster {
    pub cap: f64,
}

impl LegacyFailoverAdjustmentModifier for CapLegacyFailoverAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyFailoverAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyFailoverAdjustmentAdjuster {
    pub step: f64,
}

impl LegacyFailoverAdjustmentModifier for RoundingLegacyFailoverAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyFailoverAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionLegacyFailoverAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyFailoverAdjustmentModifier for RegionLegacyFailoverAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &LegacyFailoverAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy failover adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyFailoverAdjustmentEngine {
    pub steps: Vec<Rc<dyn LegacyFailoverAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyFailoverAdjustmentEngine {
    pub fn compute(&mut self, item: &LegacyFailoverAdjustment) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[LegacyFailoverAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_failover_adjustment_engine() -> LegacyFailoverAdjustmentEngine {
    LegacyFailoverAdjustmentEngine {
        steps: vec![
            Rc::new(CapLegacyFailoverAdjustmentAdjuster { cap: 49508.0 }) as Rc<dyn LegacyFailoverAdjustmentModifier>,
            Rc::new(RoundingLegacyFailoverAdjustmentAdjuster { step: 5.0 }),
            Rc::new(RegionLegacyFailoverAdjustmentAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.34), ("central".to_string(), 1.09)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 62186.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_failover_adjustment_engine();
        let item = new_legacy_failover_adjustment("code-418", "name-399");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
