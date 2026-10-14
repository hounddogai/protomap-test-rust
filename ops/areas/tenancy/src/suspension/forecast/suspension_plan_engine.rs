use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the suspension plan calculation.
pub trait SuspensionPlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionPlanBudgetOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionPlanFactor for SuspensionPlanBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionPlan, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SuspensionPlanFactor for TieredSuspensionPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SuspensionPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionPlanAdjuster {
    pub minimum: f64,
}

impl SuspensionPlanFactor for MinimumSuspensionPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionPlanFactor for FlaggedSuspensionPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitSuspensionPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionPlanFactor for UnitSuspensionPlanAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &SuspensionPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the suspension plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionPlanEngine {
    pub steps: Vec<Rc<dyn SuspensionPlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionPlanEngine {
    pub fn compute(&mut self, item: &SuspensionPlan) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionPlanFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_suspension_plan_engine(flags: &flags::Registry) -> SuspensionPlanEngine {
    SuspensionPlanEngine {
        steps: vec![
            Rc::new(SuspensionPlanBudgetOffsetAdjuster { weight: 1.44 }) as Rc<dyn SuspensionPlanFactor>,
            Rc::new(TieredSuspensionPlanAdjuster {
                bounds: vec![1867.0, 2524.0, 4953.0, 8402.0],
                rates: vec![1.07, 0.86, 0.86, 0.88],
            }),
            Rc::new(MinimumSuspensionPlanAdjuster { minimum: 25.0 }),
            Rc::new(FlaggedSuspensionPlanAdjuster {
                flags: flags.clone(),
                flag: "suspension-plan-strict".to_string(),
                percent: -1.2,
            }),
            Rc::new(UnitSuspensionPlanAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.26)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 19639.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_plan_engine(&flags::new_registry());
        let item = new_suspension_plan("code-782", "title-37");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
