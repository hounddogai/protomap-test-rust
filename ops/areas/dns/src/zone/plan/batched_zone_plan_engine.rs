use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched zone plan calculation.
pub trait BatchedZonePlanStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedZonePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryBatchedZonePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedZonePlanStep for CategoryBatchedZonePlanAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &BatchedZonePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedZonePlanAdjuster {
    pub minimum: f64,
}

impl BatchedZonePlanStep for MinimumBatchedZonePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedZonePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedZonePlanAdjuster {
    pub cap: f64,
}

impl BatchedZonePlanStep for CapBatchedZonePlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedZonePlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedZonePlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedZonePlanStep for FlaggedBatchedZonePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedZonePlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZonePlanMarginOffsetAdjuster {
    pub weight: f64,
}

impl BatchedZonePlanStep for BatchedZonePlanMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedZonePlan, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the batched zone plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedZonePlanEngine {
    pub steps: Vec<Rc<dyn BatchedZonePlanStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedZonePlanEngine {
    pub fn evaluate(&mut self, item: &BatchedZonePlan) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedZonePlanStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedZonePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_zone_plan_engine(flags: &flags::Registry) -> BatchedZonePlanEngine {
    BatchedZonePlanEngine {
        steps: vec![
            Rc::new(CategoryBatchedZonePlanAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.39), ("west".to_string(), 1.27)]),
            }) as Rc<dyn BatchedZonePlanStep>,
            Rc::new(MinimumBatchedZonePlanAdjuster { minimum: 41.8 }),
            Rc::new(CapBatchedZonePlanAdjuster { cap: 5346.0 }),
            Rc::new(FlaggedBatchedZonePlanAdjuster {
                flags: flags.clone(),
                flag: "batched-zone-plan-lenient".to_string(),
                percent: -0.2,
            }),
            Rc::new(BatchedZonePlanMarginOffsetAdjuster { weight: 1.16 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 86218.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_zone_plan_engine(&flags::new_registry());
        let item = new_batched_zone_plan("code-202", "label-142");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
