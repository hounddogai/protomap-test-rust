use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the regional forecast rule calculation.
pub trait RegionalForecastRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalForecastRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastRuleBudgetOffsetAdjuster {
    pub weight: f64,
}

impl RegionalForecastRuleFactor for RegionalForecastRuleBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalForecastRule, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalForecastRuleAdjuster {
    pub cap: f64,
}

impl RegionalForecastRuleFactor for CapRegionalForecastRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalForecastRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRegionalForecastRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalForecastRuleFactor for LabelRegionalForecastRuleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RegionalForecastRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the regional forecast rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalForecastRuleEngine {
    pub steps: Vec<Rc<dyn RegionalForecastRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalForecastRuleEngine {
    pub fn compute(&mut self, item: &RegionalForecastRule) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalForecastRuleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalForecastRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_forecast_rule_engine() -> RegionalForecastRuleEngine {
    RegionalForecastRuleEngine {
        steps: vec![
            Rc::new(RegionalForecastRuleBudgetOffsetAdjuster { weight: 1.4 }) as Rc<dyn RegionalForecastRuleFactor>,
            Rc::new(CapRegionalForecastRuleAdjuster { cap: 46921.0 }),
            Rc::new(LabelRegionalForecastRuleAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.35)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 49306.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_forecast_rule_engine();
        let item = new_regional_forecast_rule("code-290", "label-779");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
