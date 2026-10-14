use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional identity tier calculation.
pub trait RegionalIdentityTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalIdentityTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalIdentityTierLimitOffsetAdjuster {
    pub weight: f64,
}

impl RegionalIdentityTierAdjuster for RegionalIdentityTierLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalIdentityTier, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRegionalIdentityTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalIdentityTierAdjuster for TitleRegionalIdentityTierAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RegionalIdentityTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalIdentityTierAdjuster {
    pub step: f64,
}

impl RegionalIdentityTierAdjuster for RoundingRegionalIdentityTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalIdentityTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the regional identity tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalIdentityTierEngine {
    pub steps: Vec<Rc<dyn RegionalIdentityTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalIdentityTierEngine {
    pub fn evaluate(&mut self, item: &RegionalIdentityTier) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[RegionalIdentityTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_identity_tier_engine() -> RegionalIdentityTierEngine {
    RegionalIdentityTierEngine {
        steps: vec![
            Rc::new(RegionalIdentityTierLimitOffsetAdjuster { weight: 1.34 }) as Rc<dyn RegionalIdentityTierAdjuster>,
            Rc::new(TitleRegionalIdentityTierAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.17)]),
            }),
            Rc::new(RoundingRegionalIdentityTierAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 9639.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_identity_tier_engine();
        let item = new_regional_identity_tier("code-442", "title-842");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
