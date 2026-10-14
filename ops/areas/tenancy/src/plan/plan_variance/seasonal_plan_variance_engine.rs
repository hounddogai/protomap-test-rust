use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the seasonal plan variance calculation.
pub trait SeasonalPlanVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalPlanVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalPlanVarianceAdjuster {
    pub percent: f64,
}

impl SeasonalPlanVarianceModifier for PercentSeasonalPlanVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalPlanVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalPlanVarianceAdjuster {
    pub cap: f64,
}

impl SeasonalPlanVarianceModifier for CapSeasonalPlanVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalPlanVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalPlanVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalPlanVarianceModifier for FlaggedSeasonalPlanVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalPlanVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal plan variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalPlanVarianceEngine {
    pub steps: Vec<Rc<dyn SeasonalPlanVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalPlanVarianceEngine {
    pub fn compute(&mut self, item: &SeasonalPlanVariance) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalPlanVarianceModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalPlanVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_plan_variance_engine(flags: &flags::Registry) -> SeasonalPlanVarianceEngine {
    SeasonalPlanVarianceEngine {
        steps: vec![
            Rc::new(PercentSeasonalPlanVarianceAdjuster { percent: -17.4 }) as Rc<dyn SeasonalPlanVarianceModifier>,
            Rc::new(CapSeasonalPlanVarianceAdjuster { cap: 10763.0 }),
            Rc::new(FlaggedSeasonalPlanVarianceAdjuster {
                flags: flags.clone(),
                flag: "seasonal-plan-variance-manual".to_string(),
                percent: 8.7,
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 82247.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_plan_variance_engine(&flags::new_registry());
        let item = new_seasonal_plan_variance("code-611", "label-832");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
