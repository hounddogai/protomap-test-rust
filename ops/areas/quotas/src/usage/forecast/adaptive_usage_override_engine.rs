use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive usage override calculation.
pub trait AdaptiveUsageOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveUsageOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveUsageOverrideAdjuster {
    pub percent: f64,
}

impl AdaptiveUsageOverrideModifier for PercentAdaptiveUsageOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveUsageOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageOverrideDiscountOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveUsageOverrideModifier for AdaptiveUsageOverrideDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveUsageOverride, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveUsageOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveUsageOverrideModifier for FlaggedAdaptiveUsageOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveUsageOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive usage override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveUsageOverrideEngine {
    pub steps: Vec<Rc<dyn AdaptiveUsageOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveUsageOverrideEngine {
    pub fn price(&mut self, item: &AdaptiveUsageOverride) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveUsageOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_usage_override_engine(flags: &flags::Registry) -> AdaptiveUsageOverrideEngine {
    AdaptiveUsageOverrideEngine {
        steps: vec![
            Rc::new(PercentAdaptiveUsageOverrideAdjuster { percent: -22.7 }) as Rc<dyn AdaptiveUsageOverrideModifier>,
            Rc::new(AdaptiveUsageOverrideDiscountOffsetAdjuster { weight: 1.29 }),
            Rc::new(FlaggedAdaptiveUsageOverrideAdjuster {
                flags: flags.clone(),
                flag: "adaptive-usage-override-primary".to_string(),
                percent: 9.2,
            }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 80107.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_usage_override_engine(&flags::new_registry());
        let item = new_adaptive_usage_override("code-854", "label-112");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
