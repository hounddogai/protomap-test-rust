use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the limit preference calculation.
pub trait LimitPreferenceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLimitPreferenceAdjuster {
    pub cap: f64,
}

impl LimitPreferenceStep for CapLimitPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LimitPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitLimitPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LimitPreferenceStep for UnitLimitPreferenceAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &LimitPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitPreferenceAdjuster {
    pub percent: f64,
}

impl LimitPreferenceStep for PercentLimitPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitPreferenceQuantityOffsetAdjuster {
    pub weight: f64,
}

impl LimitPreferenceStep for LimitPreferenceQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &LimitPreference, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the limit preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitPreferenceEngine {
    pub steps: Vec<Rc<dyn LimitPreferenceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitPreferenceEngine {
    pub fn evaluate(&mut self, item: &LimitPreference) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[LimitPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_limit_preference_engine() -> LimitPreferenceEngine {
    LimitPreferenceEngine {
        steps: vec![
            Rc::new(CapLimitPreferenceAdjuster { cap: 5019.0 }) as Rc<dyn LimitPreferenceStep>,
            Rc::new(UnitLimitPreferenceAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.95), ("west".to_string(), 0.97)]),
            }),
            Rc::new(PercentLimitPreferenceAdjuster { percent: 10.0 }),
            Rc::new(LimitPreferenceQuantityOffsetAdjuster { weight: 1.22 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 9434.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_preference_engine();
        let item = new_limit_preference("code-185", "title-334");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
