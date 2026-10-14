use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rule record calculation.
pub trait RuleRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRuleRecordAdjuster {
    pub percent: f64,
}

impl RuleRecordModifier for PercentRuleRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RuleRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleRecordAdjuster {
    pub step: f64,
}

impl RuleRecordModifier for RoundingRuleRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRecordWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl RuleRecordModifier for RuleRecordWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &RuleRecord, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RuleRecordModifier for TieredRuleRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RuleRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rule record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleRecordEngine {
    pub steps: Vec<Rc<dyn RuleRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleRecordEngine {
    pub fn compute(&mut self, item: &RuleRecord) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RuleRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rule_record_engine() -> RuleRecordEngine {
    RuleRecordEngine {
        steps: vec![
            Rc::new(PercentRuleRecordAdjuster { percent: -1.5 }) as Rc<dyn RuleRecordModifier>,
            Rc::new(RoundingRuleRecordAdjuster { step: 0.25 }),
            Rc::new(RuleRecordWindowDaysOffsetAdjuster { weight: 1.53 }),
            Rc::new(TieredRuleRecordAdjuster {
                bounds: vec![1896.0, 5329.0, 9079.0],
                rates: vec![0.9, 1.1, 0.98],
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 95771.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_record_engine();
        let item = new_rule_record("code-833", "title-861");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
