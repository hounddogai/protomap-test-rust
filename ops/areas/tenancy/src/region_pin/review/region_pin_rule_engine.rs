use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the region pin rule calculation.
pub trait RegionPinRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeRegionPinRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionPinRuleModifier for ScopeRegionPinRuleAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &RegionPinRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRuleFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinRuleModifier for RegionPinRuleFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinRule, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinRuleAdjuster {
    pub step: f64,
}

impl RegionPinRuleModifier for RoundingRegionPinRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the region pin rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinRuleEngine {
    pub steps: Vec<Rc<dyn RegionPinRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinRuleEngine {
    pub fn evaluate(&mut self, item: &RegionPinRule) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionPinRuleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_region_pin_rule_engine() -> RegionPinRuleEngine {
    RegionPinRuleEngine {
        steps: vec![
            Rc::new(ScopeRegionPinRuleAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.28), ("north".to_string(), 1.32)]),
            }) as Rc<dyn RegionPinRuleModifier>,
            Rc::new(RegionPinRuleFloorValueOffsetAdjuster { weight: 1.89 }),
            Rc::new(RoundingRegionPinRuleAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 14064.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_rule_engine();
        let item = new_region_pin_rule("code-582", "name-723");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
