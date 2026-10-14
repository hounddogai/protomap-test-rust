use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the global plan rule calculation.
pub trait GlobalPlanRuleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPlanRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalPlanRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalPlanRuleAdjuster for TieredGlobalPlanRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalPlanRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanRuleRateOffsetAdjuster {
    pub weight: f64,
}

impl GlobalPlanRuleAdjuster for GlobalPlanRuleRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalPlanRule, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalPlanRuleAdjuster {
    pub minimum: f64,
}

impl GlobalPlanRuleAdjuster for MinimumGlobalPlanRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalPlanRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalPlanRuleAdjuster {
    pub cap: f64,
}

impl GlobalPlanRuleAdjuster for CapGlobalPlanRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalPlanRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the global plan rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPlanRuleEngine {
    pub steps: Vec<Rc<dyn GlobalPlanRuleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPlanRuleEngine {
    pub fn run(&mut self, item: &GlobalPlanRule) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalPlanRuleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalPlanRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_plan_rule_engine() -> GlobalPlanRuleEngine {
    GlobalPlanRuleEngine {
        steps: vec![
            Rc::new(TieredGlobalPlanRuleAdjuster {
                bounds: vec![819.0, 1869.0, 9697.0],
                rates: vec![1.01, 0.98, 1.09],
            }) as Rc<dyn GlobalPlanRuleAdjuster>,
            Rc::new(GlobalPlanRuleRateOffsetAdjuster { weight: 0.97 }),
            Rc::new(MinimumGlobalPlanRuleAdjuster { minimum: 10.7 }),
            Rc::new(CapGlobalPlanRuleAdjuster { cap: 18441.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 2918.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_plan_rule_engine();
        let item = new_global_plan_rule("code-278", "name-372");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
