use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal failover rule calculation.
pub trait SeasonalFailoverRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalFailoverRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalFailoverRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalFailoverRuleStep for TieredSeasonalFailoverRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalFailoverRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalFailoverRuleAdjuster {
    pub percent: f64,
}

impl SeasonalFailoverRuleStep for PercentSeasonalFailoverRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalFailoverRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSeasonalFailoverRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalFailoverRuleStep for NameSeasonalFailoverRuleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SeasonalFailoverRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalFailoverRuleAdjuster {
    pub cap: f64,
}

impl SeasonalFailoverRuleStep for CapSeasonalFailoverRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalFailoverRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFailoverRuleAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalFailoverRuleStep for SeasonalFailoverRuleAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalFailoverRule, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

/// Runs the seasonal failover rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalFailoverRuleEngine {
    pub steps: Vec<Rc<dyn SeasonalFailoverRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalFailoverRuleEngine {
    pub fn run(&mut self, item: &SeasonalFailoverRule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SeasonalFailoverRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_failover_rule_engine() -> SeasonalFailoverRuleEngine {
    SeasonalFailoverRuleEngine {
        steps: vec![
            Rc::new(TieredSeasonalFailoverRuleAdjuster {
                bounds: vec![1468.0, 3151.0],
                rates: vec![1.06, 0.83],
            }) as Rc<dyn SeasonalFailoverRuleStep>,
            Rc::new(PercentSeasonalFailoverRuleAdjuster { percent: 16.1 }),
            Rc::new(NameSeasonalFailoverRuleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.05), ("west".to_string(), 1.27)]),
            }),
            Rc::new(CapSeasonalFailoverRuleAdjuster { cap: 5879.0 }),
            Rc::new(SeasonalFailoverRuleAttemptsOffsetAdjuster { weight: 1.18 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 14619.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_failover_rule_engine();
        let item = new_seasonal_failover_rule("code-672", "name-475");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
