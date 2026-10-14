use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the overage plan calculation.
pub trait OveragePlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OveragePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentOveragePlanAdjuster {
    pub percent: f64,
}

impl OveragePlanModifier for PercentOveragePlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &OveragePlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOveragePlanAdjuster {
    pub step: f64,
}

impl OveragePlanModifier for RoundingOveragePlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OveragePlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedOveragePlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl OveragePlanModifier for FlaggedOveragePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &OveragePlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the overage plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OveragePlanEngine {
    pub steps: Vec<Rc<dyn OveragePlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OveragePlanEngine {
    pub fn price(&mut self, item: &OveragePlan) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn OveragePlanModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[OveragePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_overage_plan_engine(flags: &flags::Registry) -> OveragePlanEngine {
    OveragePlanEngine {
        steps: vec![
            Rc::new(PercentOveragePlanAdjuster { percent: 18.0 }) as Rc<dyn OveragePlanModifier>,
            Rc::new(RoundingOveragePlanAdjuster { step: 0.05 }),
            Rc::new(FlaggedOveragePlanAdjuster {
                flags: flags.clone(),
                flag: "overage-plan-pending".to_string(),
                percent: 9.2,
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 15036.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_plan_engine(&flags::new_registry());
        let item = new_overage_plan("code-235", "title-951");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
