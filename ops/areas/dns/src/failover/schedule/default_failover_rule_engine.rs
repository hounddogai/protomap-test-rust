use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default failover rule calculation.
pub trait DefaultFailoverRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultFailoverRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFailoverRuleRevisionOffsetAdjuster {
    pub weight: f64,
}

impl DefaultFailoverRuleStep for DefaultFailoverRuleRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultFailoverRule, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultFailoverRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultFailoverRuleStep for TieredDefaultFailoverRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultFailoverRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDefaultFailoverRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultFailoverRuleStep for NameDefaultFailoverRuleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DefaultFailoverRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultFailoverRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultFailoverRuleStep for FlaggedDefaultFailoverRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultFailoverRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultFailoverRuleAdjuster {
    pub cap: f64,
}

impl DefaultFailoverRuleStep for CapDefaultFailoverRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultFailoverRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the default failover rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultFailoverRuleEngine {
    pub steps: Vec<Rc<dyn DefaultFailoverRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultFailoverRuleEngine {
    pub fn price(&mut self, item: &DefaultFailoverRule) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultFailoverRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_failover_rule_engine(flags: &flags::Registry) -> DefaultFailoverRuleEngine {
    DefaultFailoverRuleEngine {
        steps: vec![
            Rc::new(DefaultFailoverRuleRevisionOffsetAdjuster { weight: 1.92 }) as Rc<dyn DefaultFailoverRuleStep>,
            Rc::new(TieredDefaultFailoverRuleAdjuster {
                bounds: vec![5946.0, 8774.0],
                rates: vec![0.97, 0.99],
            }),
            Rc::new(NameDefaultFailoverRuleAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.03),
                    ("east".to_string(), 1.3),
                    ("online".to_string(), 1.35),
                ]),
            }),
            Rc::new(FlaggedDefaultFailoverRuleAdjuster {
                flags: flags.clone(),
                flag: "default-failover-rule-scheduled".to_string(),
                percent: 4.5,
            }),
            Rc::new(CapDefaultFailoverRuleAdjuster { cap: 48610.0 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 12761.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_failover_rule_engine(&flags::new_registry());
        let item = new_default_failover_rule("code-201", "name-729");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
