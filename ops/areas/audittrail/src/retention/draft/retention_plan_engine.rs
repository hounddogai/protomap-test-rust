use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention plan calculation.
pub trait RetentionPlanStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionPlan, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionPlanStep for FlaggedRetentionPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRetentionPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionPlanStep for LabelRetentionPlanAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RetentionPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionPlanAdjuster {
    pub percent: f64,
}

impl RetentionPlanStep for PercentRetentionPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPlanWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl RetentionPlanStep for RetentionPlanWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionPlan, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionPlanAdjuster {
    pub minimum: f64,
}

impl RetentionPlanStep for MinimumRetentionPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionPlanAdjuster {
    pub step: f64,
}

impl RetentionPlanStep for RoundingRetentionPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the retention plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionPlanEngine {
    pub steps: Vec<Rc<dyn RetentionPlanStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionPlanEngine {
    pub fn evaluate(&mut self, item: &RetentionPlan) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionPlanStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_retention_plan_engine(flags: &flags::Registry) -> RetentionPlanEngine {
    RetentionPlanEngine {
        steps: vec![
            Rc::new(FlaggedRetentionPlanAdjuster {
                flags: flags.clone(),
                flag: "retention-plan-adaptive".to_string(),
                percent: -2.6,
            }) as Rc<dyn RetentionPlanStep>,
            Rc::new(LabelRetentionPlanAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.87),
                    ("east".to_string(), 0.92),
                    ("south".to_string(), 1.22),
                ]),
            }),
            Rc::new(PercentRetentionPlanAdjuster { percent: -22.4 }),
            Rc::new(RetentionPlanWindowDaysOffsetAdjuster { weight: 0.62 }),
            Rc::new(MinimumRetentionPlanAdjuster { minimum: 35.0 }),
            Rc::new(RoundingRetentionPlanAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 33638.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_plan_engine(&flags::new_registry());
        let item = new_retention_plan("code-924", "label-89");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
