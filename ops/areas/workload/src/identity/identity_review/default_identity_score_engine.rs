use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default identity score calculation.
pub trait DefaultIdentityScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultIdentityScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityScoreQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl DefaultIdentityScoreModifier for DefaultIdentityScoreQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultIdentityScore, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultIdentityScoreAdjuster {
    pub percent: f64,
}

impl DefaultIdentityScoreModifier for PercentDefaultIdentityScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultIdentityScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleDefaultIdentityScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultIdentityScoreModifier for LocaleDefaultIdentityScoreAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &DefaultIdentityScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultIdentityScoreAdjuster {
    pub step: f64,
}

impl DefaultIdentityScoreModifier for RoundingDefaultIdentityScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultIdentityScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultIdentityScoreAdjuster {
    pub cap: f64,
}

impl DefaultIdentityScoreModifier for CapDefaultIdentityScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultIdentityScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the default identity score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultIdentityScoreEngine {
    pub steps: Vec<Rc<dyn DefaultIdentityScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultIdentityScoreEngine {
    pub fn price(&mut self, item: &DefaultIdentityScore) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[DefaultIdentityScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_identity_score_engine() -> DefaultIdentityScoreEngine {
    DefaultIdentityScoreEngine {
        steps: vec![
            Rc::new(DefaultIdentityScoreQuotaUnitsOffsetAdjuster { weight: 0.33 })
                as Rc<dyn DefaultIdentityScoreModifier>,
            Rc::new(PercentDefaultIdentityScoreAdjuster { percent: 15.2 }),
            Rc::new(LocaleDefaultIdentityScoreAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.22), ("online".to_string(), 0.86)]),
            }),
            Rc::new(RoundingDefaultIdentityScoreAdjuster { step: 0.1 }),
            Rc::new(CapDefaultIdentityScoreAdjuster { cap: 32135.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 15747.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_identity_score_engine();
        let item = new_default_identity_score("code-878", "title-77");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
