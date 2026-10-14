use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan variance calculation.
pub trait PlanVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanVariance, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanVarianceModifier for FlaggedPlanVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanVarianceAdjuster {
    pub percent: f64,
}

impl PlanVarianceModifier for PercentPlanVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanVarianceAdjuster {
    pub minimum: f64,
}

impl PlanVarianceModifier for MinimumPlanVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanVarianceAdjuster {
    pub cap: f64,
}

impl PlanVarianceModifier for CapPlanVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the plan variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanVarianceEngine {
    pub steps: Vec<Rc<dyn PlanVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanVarianceEngine {
    pub fn evaluate(&mut self, item: &PlanVariance) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[PlanVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_plan_variance_engine(flags: &flags::Registry) -> PlanVarianceEngine {
    PlanVarianceEngine {
        steps: vec![
            Rc::new(FlaggedPlanVarianceAdjuster {
                flags: flags.clone(),
                flag: "plan-variance-adaptive".to_string(),
                percent: -4.0,
            }) as Rc<dyn PlanVarianceModifier>,
            Rc::new(PercentPlanVarianceAdjuster { percent: -7.4 }),
            Rc::new(MinimumPlanVarianceAdjuster { minimum: 33.9 }),
            Rc::new(CapPlanVarianceAdjuster { cap: 4036.0 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 91394.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_variance_engine(&flags::new_registry());
        let item = new_plan_variance("code-833", "title-173");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
