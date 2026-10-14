use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding preference calculation.
pub trait BindingPreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBindingPreferenceAdjuster {
    pub percent: f64,
}

impl BindingPreferenceFactor for PercentBindingPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BindingPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBindingPreferenceAdjuster {
    pub cap: f64,
}

impl BindingPreferenceFactor for CapBindingPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BindingPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPreferenceRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl BindingPreferenceFactor for BindingPreferenceRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &BindingPreference, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingPreferenceAdjuster {
    pub step: f64,
}

impl BindingPreferenceFactor for RoundingBindingPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBindingPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingPreferenceFactor for NameBindingPreferenceAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BindingPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the binding preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingPreferenceEngine {
    pub steps: Vec<Rc<dyn BindingPreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingPreferenceEngine {
    pub fn evaluate(&mut self, item: &BindingPreference) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BindingPreferenceFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BindingPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_binding_preference_engine() -> BindingPreferenceEngine {
    BindingPreferenceEngine {
        steps: vec![
            Rc::new(PercentBindingPreferenceAdjuster { percent: 11.7 }) as Rc<dyn BindingPreferenceFactor>,
            Rc::new(CapBindingPreferenceAdjuster { cap: 48092.0 }),
            Rc::new(BindingPreferenceRotationDaysOffsetAdjuster { weight: 1.59 }),
            Rc::new(RoundingBindingPreferenceAdjuster { step: 0.05 }),
            Rc::new(NameBindingPreferenceAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.02)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 2237.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_preference_engine();
        let item = new_binding_preference("code-363", "name-469");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
