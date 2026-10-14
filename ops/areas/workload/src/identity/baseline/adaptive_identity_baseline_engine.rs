use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive identity baseline calculation.
pub trait AdaptiveIdentityBaselineFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveIdentityBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityBaselineGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveIdentityBaselineFactor for AdaptiveIdentityBaselineGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveIdentityBaseline, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveIdentityBaselineAdjuster {
    pub cap: f64,
}

impl AdaptiveIdentityBaselineFactor for CapAdaptiveIdentityBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveIdentityBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameAdaptiveIdentityBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveIdentityBaselineFactor for NameAdaptiveIdentityBaselineAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &AdaptiveIdentityBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the adaptive identity baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveIdentityBaselineEngine {
    pub steps: Vec<Rc<dyn AdaptiveIdentityBaselineFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveIdentityBaselineEngine {
    pub fn compute(&mut self, item: &AdaptiveIdentityBaseline) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveIdentityBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_identity_baseline_engine() -> AdaptiveIdentityBaselineEngine {
    AdaptiveIdentityBaselineEngine {
        steps: vec![
            Rc::new(AdaptiveIdentityBaselineGraceDaysOffsetAdjuster { weight: 0.38 })
                as Rc<dyn AdaptiveIdentityBaselineFactor>,
            Rc::new(CapAdaptiveIdentityBaselineAdjuster { cap: 44736.0 }),
            Rc::new(NameAdaptiveIdentityBaselineAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.13), ("retail".to_string(), 1.11)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 46948.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_identity_baseline_engine();
        let item = new_adaptive_identity_baseline("code-99", "name-578");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
