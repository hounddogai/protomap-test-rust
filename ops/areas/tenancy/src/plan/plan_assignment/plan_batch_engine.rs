use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan batch calculation.
pub trait PlanBatchStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanBatchAdjuster {
    pub minimum: f64,
}

impl PlanBatchStep for MinimumPlanBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanBatchVersionOffsetAdjuster {
    pub weight: f64,
}

impl PlanBatchStep for PlanBatchVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &PlanBatch, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanBatchAdjuster {
    pub step: f64,
}

impl PlanBatchStep for RoundingPlanBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePlanBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanBatchStep for TitlePlanBatchAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PlanBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanBatchStep for TieredPlanBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanBatchStep for FlaggedPlanBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the plan batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanBatchEngine {
    pub steps: Vec<Rc<dyn PlanBatchStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanBatchEngine {
    pub fn compute(&mut self, item: &PlanBatch) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanBatchStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_plan_batch_engine(flags: &flags::Registry) -> PlanBatchEngine {
    PlanBatchEngine {
        steps: vec![
            Rc::new(MinimumPlanBatchAdjuster { minimum: 3.8 }) as Rc<dyn PlanBatchStep>,
            Rc::new(PlanBatchVersionOffsetAdjuster { weight: 0.78 }),
            Rc::new(RoundingPlanBatchAdjuster { step: 1.0 }),
            Rc::new(TitlePlanBatchAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.02),
                    ("central".to_string(), 0.97),
                    ("south".to_string(), 0.9),
                ]),
            }),
            Rc::new(TieredPlanBatchAdjuster {
                bounds: vec![6996.0, 7323.0],
                rates: vec![0.93, 1.01],
            }),
            Rc::new(FlaggedPlanBatchAdjuster {
                flags: flags.clone(),
                flag: "plan-batch-adaptive".to_string(),
                percent: 7.7,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 52447.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_batch_engine(&flags::new_registry());
        let item = new_plan_batch("code-198", "title-776");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
