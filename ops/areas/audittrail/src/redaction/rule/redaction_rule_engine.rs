use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the redaction rule calculation.
pub trait RedactionRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRuleEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl RedactionRuleStep for RedactionRuleEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionRule, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionRuleStep for TieredRedactionRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRedactionRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionRuleStep for CountryRedactionRuleAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RedactionRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the redaction rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionRuleEngine {
    pub steps: Vec<Rc<dyn RedactionRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionRuleEngine {
    pub fn evaluate(&mut self, item: &RedactionRule) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[RedactionRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_redaction_rule_engine() -> RedactionRuleEngine {
    RedactionRuleEngine {
        steps: vec![
            Rc::new(RedactionRuleEndsAtOffsetAdjuster { weight: 1.58 }) as Rc<dyn RedactionRuleStep>,
            Rc::new(TieredRedactionRuleAdjuster {
                bounds: vec![1489.0, 8380.0, 8889.0, 9167.0],
                rates: vec![0.94, 0.88, 1.16, 1.13],
            }),
            Rc::new(CountryRedactionRuleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.12), ("south".to_string(), 1.1)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 65597.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_rule_engine();
        let item = new_redaction_rule("code-129", "title-820");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
