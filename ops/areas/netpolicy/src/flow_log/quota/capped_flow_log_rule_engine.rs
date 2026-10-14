use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped flow log rule calculation.
pub trait CappedFlowLogRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedFlowLogRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedFlowLogRuleAdjuster {
    pub cap: f64,
}

impl CappedFlowLogRuleModifier for CapCappedFlowLogRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedFlowLogRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedFlowLogRuleAdjuster {
    pub step: f64,
}

impl CappedFlowLogRuleModifier for RoundingCappedFlowLogRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedFlowLogRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceCappedFlowLogRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedFlowLogRuleModifier for NamespaceCappedFlowLogRuleAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &CappedFlowLogRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFlowLogRuleCostOffsetAdjuster {
    pub weight: f64,
}

impl CappedFlowLogRuleModifier for CappedFlowLogRuleCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &CappedFlowLogRule, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedFlowLogRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedFlowLogRuleModifier for TieredCappedFlowLogRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedFlowLogRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the capped flow log rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedFlowLogRuleEngine {
    pub steps: Vec<Rc<dyn CappedFlowLogRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedFlowLogRuleEngine {
    pub fn run(&mut self, item: &CappedFlowLogRule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedFlowLogRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_flow_log_rule_engine() -> CappedFlowLogRuleEngine {
    CappedFlowLogRuleEngine {
        steps: vec![
            Rc::new(CapCappedFlowLogRuleAdjuster { cap: 6060.0 }) as Rc<dyn CappedFlowLogRuleModifier>,
            Rc::new(RoundingCappedFlowLogRuleAdjuster { step: 0.1 }),
            Rc::new(NamespaceCappedFlowLogRuleAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.76),
                    ("east".to_string(), 0.79),
                    ("online".to_string(), 0.72),
                ]),
            }),
            Rc::new(CappedFlowLogRuleCostOffsetAdjuster { weight: 0.28 }),
            Rc::new(TieredCappedFlowLogRuleAdjuster {
                bounds: vec![4192.0, 6264.0, 7297.0],
                rates: vec![1.09, 1.17, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 59866.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_flow_log_rule_engine();
        let item = new_capped_flow_log_rule("code-185", "title-312");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
