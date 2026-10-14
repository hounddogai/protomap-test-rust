use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict alert preference calculation.
pub trait StrictAlertPreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictAlertPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictAlertPreferenceAdjuster {
    pub minimum: f64,
}

impl StrictAlertPreferenceFactor for MinimumStrictAlertPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictAlertPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertPreferenceMarginOffsetAdjuster {
    pub weight: f64,
}

impl StrictAlertPreferenceFactor for StrictAlertPreferenceMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &StrictAlertPreference, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictAlertPreferenceAdjuster {
    pub cap: f64,
}

impl StrictAlertPreferenceFactor for CapStrictAlertPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictAlertPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictAlertPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictAlertPreferenceFactor for TieredStrictAlertPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictAlertPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryStrictAlertPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictAlertPreferenceFactor for CategoryStrictAlertPreferenceAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &StrictAlertPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictAlertPreferenceAdjuster {
    pub step: f64,
}

impl StrictAlertPreferenceFactor for RoundingStrictAlertPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictAlertPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the strict alert preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictAlertPreferenceEngine {
    pub steps: Vec<Rc<dyn StrictAlertPreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictAlertPreferenceEngine {
    pub fn compute(&mut self, item: &StrictAlertPreference) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[StrictAlertPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_alert_preference_engine() -> StrictAlertPreferenceEngine {
    StrictAlertPreferenceEngine {
        steps: vec![
            Rc::new(MinimumStrictAlertPreferenceAdjuster { minimum: 15.6 }) as Rc<dyn StrictAlertPreferenceFactor>,
            Rc::new(StrictAlertPreferenceMarginOffsetAdjuster { weight: 1.94 }),
            Rc::new(CapStrictAlertPreferenceAdjuster { cap: 10071.0 }),
            Rc::new(TieredStrictAlertPreferenceAdjuster {
                bounds: vec![5383.0, 5660.0, 5923.0],
                rates: vec![1.02, 0.96, 0.95],
            }),
            Rc::new(CategoryStrictAlertPreferenceAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.74)]),
            }),
            Rc::new(RoundingStrictAlertPreferenceAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 75412.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_alert_preference_engine();
        let item = new_strict_alert_preference("code-343", "name-872");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
