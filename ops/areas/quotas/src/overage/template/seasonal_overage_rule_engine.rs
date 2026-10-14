use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal overage rule calculation.
pub trait SeasonalOverageRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalOverageRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageRuleAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalOverageRuleStep for SeasonalOverageRuleAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalOverageRule, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalOverageRuleAdjuster {
    pub step: f64,
}

impl SeasonalOverageRuleStep for RoundingSeasonalOverageRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalOverageRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountrySeasonalOverageRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalOverageRuleStep for CountrySeasonalOverageRuleAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &SeasonalOverageRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the seasonal overage rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalOverageRuleEngine {
    pub steps: Vec<Rc<dyn SeasonalOverageRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalOverageRuleEngine {
    pub fn run(&mut self, item: &SeasonalOverageRule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SeasonalOverageRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_overage_rule_engine() -> SeasonalOverageRuleEngine {
    SeasonalOverageRuleEngine {
        steps: vec![
            Rc::new(SeasonalOverageRuleAttemptsOffsetAdjuster { weight: 0.81 }) as Rc<dyn SeasonalOverageRuleStep>,
            Rc::new(RoundingSeasonalOverageRuleAdjuster { step: 5.0 }),
            Rc::new(CountrySeasonalOverageRuleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.04), ("west".to_string(), 1.32)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 50482.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_overage_rule_engine();
        let item = new_seasonal_overage_rule("code-458", "name-582");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
