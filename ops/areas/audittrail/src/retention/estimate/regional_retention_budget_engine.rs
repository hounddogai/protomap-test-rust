use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional retention budget calculation.
pub trait RegionalRetentionBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalRetentionBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalRetentionBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalRetentionBudgetModifier for FlaggedRegionalRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalRetentionBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalRetentionBudgetAdjuster {
    pub percent: f64,
}

impl RegionalRetentionBudgetModifier for PercentRegionalRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalRetentionBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalRetentionBudgetAdjuster {
    pub cap: f64,
}

impl RegionalRetentionBudgetModifier for CapRegionalRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalRetentionBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionBudgetMinimumOffsetAdjuster {
    pub weight: f64,
}

impl RegionalRetentionBudgetModifier for RegionalRetentionBudgetMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalRetentionBudget, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalRetentionBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalRetentionBudgetModifier for TieredRegionalRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalRetentionBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional retention budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalRetentionBudgetEngine {
    pub steps: Vec<Rc<dyn RegionalRetentionBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalRetentionBudgetEngine {
    pub fn run(&mut self, item: &RegionalRetentionBudget) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalRetentionBudgetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalRetentionBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_retention_budget_engine(flags: &flags::Registry) -> RegionalRetentionBudgetEngine {
    RegionalRetentionBudgetEngine {
        steps: vec![
            Rc::new(FlaggedRegionalRetentionBudgetAdjuster {
                flags: flags.clone(),
                flag: "regional-retention-budget-fallback".to_string(),
                percent: 9.2,
            }) as Rc<dyn RegionalRetentionBudgetModifier>,
            Rc::new(PercentRegionalRetentionBudgetAdjuster { percent: 9.0 }),
            Rc::new(CapRegionalRetentionBudgetAdjuster { cap: 40849.0 }),
            Rc::new(RegionalRetentionBudgetMinimumOffsetAdjuster { weight: 0.26 }),
            Rc::new(TieredRegionalRetentionBudgetAdjuster {
                bounds: vec![8991.0, 9672.0],
                rates: vec![0.87, 0.86],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 19158.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_retention_budget_engine(&flags::new_registry());
        let item = new_regional_retention_budget("code-4", "name-224");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
