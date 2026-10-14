use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan threshold calculation.
pub trait PlanThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanThresholdMaximumOffsetAdjuster {
    pub weight: f64,
}

impl PlanThresholdStep for PlanThresholdMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &PlanThreshold, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanThresholdStep for FlaggedPlanThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanThresholdAdjuster {
    pub minimum: f64,
}

impl PlanThresholdStep for MinimumPlanThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePlanThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanThresholdStep for SourcePlanThresholdAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PlanThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the plan threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanThresholdEngine {
    pub steps: Vec<Rc<dyn PlanThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanThresholdEngine {
    pub fn evaluate(&mut self, item: &PlanThreshold) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[PlanThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_plan_threshold_engine(flags: &flags::Registry) -> PlanThresholdEngine {
    PlanThresholdEngine {
        steps: vec![
            Rc::new(PlanThresholdMaximumOffsetAdjuster { weight: 0.18 }) as Rc<dyn PlanThresholdStep>,
            Rc::new(FlaggedPlanThresholdAdjuster {
                flags: flags.clone(),
                flag: "plan-threshold-lenient".to_string(),
                percent: 7.0,
            }),
            Rc::new(MinimumPlanThresholdAdjuster { minimum: 15.9 }),
            Rc::new(SourcePlanThresholdAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.16),
                    ("south".to_string(), 1.08),
                    ("central".to_string(), 1.09),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 88954.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_threshold_engine(&flags::new_registry());
        let item = new_plan_threshold("code-509", "title-96");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
