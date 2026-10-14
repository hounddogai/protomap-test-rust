use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient limit rule calculation.
pub trait LenientLimitRuleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientLimitRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientLimitRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientLimitRuleAdjuster for TieredLenientLimitRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientLimitRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientLimitRuleAdjuster {
    pub step: f64,
}

impl LenientLimitRuleAdjuster for RoundingLenientLimitRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientLimitRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientLimitRuleAdjuster {
    pub minimum: f64,
}

impl LenientLimitRuleAdjuster for MinimumLenientLimitRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientLimitRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient limit rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientLimitRuleEngine {
    pub steps: Vec<Rc<dyn LenientLimitRuleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientLimitRuleEngine {
    pub fn price(&mut self, item: &LenientLimitRule) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientLimitRuleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientLimitRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_limit_rule_engine() -> LenientLimitRuleEngine {
    LenientLimitRuleEngine {
        steps: vec![
            Rc::new(TieredLenientLimitRuleAdjuster {
                bounds: vec![1309.0, 2429.0, 4347.0, 6478.0],
                rates: vec![0.99, 0.95, 0.82, 0.99],
            }) as Rc<dyn LenientLimitRuleAdjuster>,
            Rc::new(RoundingLenientLimitRuleAdjuster { step: 0.05 }),
            Rc::new(MinimumLenientLimitRuleAdjuster { minimum: 39.0 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 61064.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_limit_rule_engine();
        let item = new_lenient_limit_rule("code-858", "label-954");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
