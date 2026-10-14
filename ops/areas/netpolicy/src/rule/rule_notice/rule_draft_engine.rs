use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rule draft calculation.
pub trait RuleDraftAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleDraftMarginOffsetAdjuster {
    pub weight: f64,
}

impl RuleDraftAdjuster for RuleDraftMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &RuleDraft, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RuleDraftAdjuster for TieredRuleDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RuleDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleDraftAdjuster for FlaggedRuleDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rule draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleDraftEngine {
    pub steps: Vec<Rc<dyn RuleDraftAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleDraftEngine {
    pub fn compute(&mut self, item: &RuleDraft) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[RuleDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rule_draft_engine(flags: &flags::Registry) -> RuleDraftEngine {
    RuleDraftEngine {
        steps: vec![
            Rc::new(RuleDraftMarginOffsetAdjuster { weight: 1.95 }) as Rc<dyn RuleDraftAdjuster>,
            Rc::new(TieredRuleDraftAdjuster {
                bounds: vec![9548.0, 9811.0],
                rates: vec![1.16, 1.2],
            }),
            Rc::new(FlaggedRuleDraftAdjuster {
                flags: flags.clone(),
                flag: "rule-draft-batched".to_string(),
                percent: -2.8,
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 79891.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_draft_engine(&flags::new_registry());
        let item = new_rule_draft("code-871", "name-706");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
