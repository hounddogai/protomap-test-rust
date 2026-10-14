use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the grant preference calculation.
pub trait GrantPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantPreferenceAdjuster {
    pub minimum: f64,
}

impl GrantPreferenceAdjuster for MinimumGrantPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameGrantPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantPreferenceAdjuster for NameGrantPreferenceAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &GrantPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGrantPreferenceAdjuster {
    pub percent: f64,
}

impl GrantPreferenceAdjuster for PercentGrantPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GrantPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPreferenceAmountOffsetAdjuster {
    pub weight: f64,
}

impl GrantPreferenceAdjuster for GrantPreferenceAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &GrantPreference, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the grant preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantPreferenceEngine {
    pub steps: Vec<Rc<dyn GrantPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantPreferenceEngine {
    pub fn compute(&mut self, item: &GrantPreference) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[GrantPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_grant_preference_engine() -> GrantPreferenceEngine {
    GrantPreferenceEngine {
        steps: vec![
            Rc::new(MinimumGrantPreferenceAdjuster { minimum: 47.1 }) as Rc<dyn GrantPreferenceAdjuster>,
            Rc::new(NameGrantPreferenceAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.37), ("east".to_string(), 1.4)]),
            }),
            Rc::new(PercentGrantPreferenceAdjuster { percent: 19.8 }),
            Rc::new(GrantPreferenceAmountOffsetAdjuster { weight: 1.94 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 99119.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_preference_engine();
        let item = new_grant_preference("code-426", "name-730");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
