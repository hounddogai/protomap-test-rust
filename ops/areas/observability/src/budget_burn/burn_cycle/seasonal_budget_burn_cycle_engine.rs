use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal budget burn cycle calculation.
pub trait SeasonalBudgetBurnCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalBudgetBurnCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalBudgetBurnCycleAdjuster {
    pub minimum: f64,
}

impl SeasonalBudgetBurnCycleFactor for MinimumSeasonalBudgetBurnCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitSeasonalBudgetBurnCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalBudgetBurnCycleFactor for UnitSeasonalBudgetBurnCycleAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &SeasonalBudgetBurnCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalBudgetBurnCycleAdjuster {
    pub cap: f64,
}

impl SeasonalBudgetBurnCycleFactor for CapSeasonalBudgetBurnCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalBudgetBurnCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalBudgetBurnCycleFactor for TieredSeasonalBudgetBurnCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalBudgetBurnCycleAdjuster {
    pub percent: f64,
}

impl SeasonalBudgetBurnCycleFactor for PercentSeasonalBudgetBurnCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalBudgetBurnCycleAdjuster {
    pub step: f64,
}

impl SeasonalBudgetBurnCycleFactor for RoundingSeasonalBudgetBurnCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the seasonal budget burn cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalBudgetBurnCycleEngine {
    pub steps: Vec<Rc<dyn SeasonalBudgetBurnCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalBudgetBurnCycleEngine {
    pub fn evaluate(&mut self, item: &SeasonalBudgetBurnCycle) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalBudgetBurnCycleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalBudgetBurnCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_budget_burn_cycle_engine() -> SeasonalBudgetBurnCycleEngine {
    SeasonalBudgetBurnCycleEngine {
        steps: vec![
            Rc::new(MinimumSeasonalBudgetBurnCycleAdjuster { minimum: 28.1 }) as Rc<dyn SeasonalBudgetBurnCycleFactor>,
            Rc::new(UnitSeasonalBudgetBurnCycleAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.19),
                    ("retail".to_string(), 1.01),
                    ("west".to_string(), 0.82),
                ]),
            }),
            Rc::new(CapSeasonalBudgetBurnCycleAdjuster { cap: 41003.0 }),
            Rc::new(TieredSeasonalBudgetBurnCycleAdjuster {
                bounds: vec![1666.0, 6144.0, 8240.0, 9222.0],
                rates: vec![0.9, 0.95, 0.99, 0.83],
            }),
            Rc::new(PercentSeasonalBudgetBurnCycleAdjuster { percent: -16.1 }),
            Rc::new(RoundingSeasonalBudgetBurnCycleAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 52661.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_budget_burn_cycle_engine();
        let item = new_seasonal_budget_burn_cycle("code-582", "name-468");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
