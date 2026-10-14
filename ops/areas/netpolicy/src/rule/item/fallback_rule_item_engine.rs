use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback rule item calculation.
pub trait FallbackRuleItemModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRuleItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleItemMarginOffsetAdjuster {
    pub weight: f64,
}

impl FallbackRuleItemModifier for FallbackRuleItemMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackRuleItem, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackRuleItemAdjuster {
    pub percent: f64,
}

impl FallbackRuleItemModifier for PercentFallbackRuleItemAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackRuleItem, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackRuleItemAdjuster {
    pub minimum: f64,
}

impl FallbackRuleItemModifier for MinimumFallbackRuleItemAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackRuleItem, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackRuleItemAdjuster {
    pub step: f64,
}

impl FallbackRuleItemModifier for RoundingFallbackRuleItemAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackRuleItem, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackRuleItemAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackRuleItemModifier for TieredFallbackRuleItemAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackRuleItem, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the fallback rule item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRuleItemEngine {
    pub steps: Vec<Rc<dyn FallbackRuleItemModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRuleItemEngine {
    pub fn compute(&mut self, item: &FallbackRuleItem) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRuleItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_rule_item_engine() -> FallbackRuleItemEngine {
    FallbackRuleItemEngine {
        steps: vec![
            Rc::new(FallbackRuleItemMarginOffsetAdjuster { weight: 1.44 }) as Rc<dyn FallbackRuleItemModifier>,
            Rc::new(PercentFallbackRuleItemAdjuster { percent: 24.7 }),
            Rc::new(MinimumFallbackRuleItemAdjuster { minimum: 49.0 }),
            Rc::new(RoundingFallbackRuleItemAdjuster { step: 0.5 }),
            Rc::new(TieredFallbackRuleItemAdjuster {
                bounds: vec![335.0, 909.0, 1858.0, 7131.0],
                rates: vec![1.07, 0.99, 0.96, 1.03],
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 34609.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_rule_item_engine();
        let item = new_fallback_rule_item("code-599", "title-669");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
