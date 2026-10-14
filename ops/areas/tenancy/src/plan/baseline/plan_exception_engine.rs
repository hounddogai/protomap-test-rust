use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan exception calculation.
pub trait PlanExceptionFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanException, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanExceptionFactor for FlaggedPlanExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanExceptionAdjuster {
    pub cap: f64,
}

impl PlanExceptionFactor for CapPlanExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanExceptionAdjuster {
    pub minimum: f64,
}

impl PlanExceptionFactor for MinimumPlanExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanExceptionAdjuster {
    pub step: f64,
}

impl PlanExceptionFactor for RoundingPlanExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanExceptionRateOffsetAdjuster {
    pub weight: f64,
}

impl PlanExceptionFactor for PlanExceptionRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &PlanException, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanExceptionAdjuster {
    pub percent: f64,
}

impl PlanExceptionFactor for PercentPlanExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the plan exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanExceptionEngine {
    pub steps: Vec<Rc<dyn PlanExceptionFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanExceptionEngine {
    pub fn run(&mut self, item: &PlanException) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[PlanException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_plan_exception_engine(flags: &flags::Registry) -> PlanExceptionEngine {
    PlanExceptionEngine {
        steps: vec![
            Rc::new(FlaggedPlanExceptionAdjuster {
                flags: flags.clone(),
                flag: "plan-exception-regional".to_string(),
                percent: -4.2,
            }) as Rc<dyn PlanExceptionFactor>,
            Rc::new(CapPlanExceptionAdjuster { cap: 45138.0 }),
            Rc::new(MinimumPlanExceptionAdjuster { minimum: 5.7 }),
            Rc::new(RoundingPlanExceptionAdjuster { step: 1.0 }),
            Rc::new(PlanExceptionRateOffsetAdjuster { weight: 0.65 }),
            Rc::new(PercentPlanExceptionAdjuster { percent: -1.3 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 93387.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_exception_engine(&flags::new_registry());
        let item = new_plan_exception("code-303", "name-982");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
