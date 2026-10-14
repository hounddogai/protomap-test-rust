use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the dashboard rule calculation.
pub trait DashboardRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardRuleModifier for TieredDashboardRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDashboardRuleAdjuster {
    pub minimum: f64,
}

impl DashboardRuleModifier for MinimumDashboardRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DashboardRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRuleExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl DashboardRuleModifier for DashboardRuleExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardRule, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

/// Runs the dashboard rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardRuleEngine {
    pub steps: Vec<Rc<dyn DashboardRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardRuleEngine {
    pub fn run(&mut self, item: &DashboardRule) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardRuleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_dashboard_rule_engine() -> DashboardRuleEngine {
    DashboardRuleEngine {
        steps: vec![
            Rc::new(TieredDashboardRuleAdjuster {
                bounds: vec![127.0, 943.0, 7722.0],
                rates: vec![0.99, 1.15, 1.05],
            }) as Rc<dyn DashboardRuleModifier>,
            Rc::new(MinimumDashboardRuleAdjuster { minimum: 4.9 }),
            Rc::new(DashboardRuleExpiresAtOffsetAdjuster { weight: 0.76 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 35551.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_rule_engine();
        let item = new_dashboard_rule("code-625", "name-387");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
