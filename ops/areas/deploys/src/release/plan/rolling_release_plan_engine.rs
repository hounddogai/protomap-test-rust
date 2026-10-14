use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rolling release plan calculation.
pub trait RollingReleasePlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingReleasePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingReleasePlanAdjuster {
    pub cap: f64,
}

impl RollingReleasePlanModifier for CapRollingReleasePlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingReleasePlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRollingReleasePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingReleasePlanModifier for LocaleRollingReleasePlanAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RollingReleasePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingReleasePlanAdjuster {
    pub percent: f64,
}

impl RollingReleasePlanModifier for PercentRollingReleasePlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingReleasePlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingReleasePlanAdjuster {
    pub minimum: f64,
}

impl RollingReleasePlanModifier for MinimumRollingReleasePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingReleasePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingReleasePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingReleasePlanModifier for TieredRollingReleasePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingReleasePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling release plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingReleasePlanEngine {
    pub steps: Vec<Rc<dyn RollingReleasePlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingReleasePlanEngine {
    pub fn compute(&mut self, item: &RollingReleasePlan) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingReleasePlanModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingReleasePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_release_plan_engine() -> RollingReleasePlanEngine {
    RollingReleasePlanEngine {
        steps: vec![
            Rc::new(CapRollingReleasePlanAdjuster { cap: 4722.0 }) as Rc<dyn RollingReleasePlanModifier>,
            Rc::new(LocaleRollingReleasePlanAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.32), ("central".to_string(), 1.06)]),
            }),
            Rc::new(PercentRollingReleasePlanAdjuster { percent: -19.5 }),
            Rc::new(MinimumRollingReleasePlanAdjuster { minimum: 21.3 }),
            Rc::new(TieredRollingReleasePlanAdjuster {
                bounds: vec![1173.0, 7571.0],
                rates: vec![1.04, 0.98],
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 41962.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_release_plan_engine();
        let item = new_rolling_release_plan("code-729", "label-849");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
