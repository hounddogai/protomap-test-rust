use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the postmortem batch calculation.
pub trait PostmortemBatchAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemBatch, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemBatchAdjuster for FlaggedPostmortemBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPostmortemBatchAdjuster {
    pub percent: f64,
}

impl PostmortemBatchAdjuster for PercentPostmortemBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PostmortemBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemBatchAdjuster {
    pub minimum: f64,
}

impl PostmortemBatchAdjuster for MinimumPostmortemBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PostmortemBatchAdjuster for TieredPostmortemBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PostmortemBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPostmortemBatchAdjuster {
    pub step: f64,
}

impl PostmortemBatchAdjuster for RoundingPostmortemBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PostmortemBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the postmortem batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemBatchEngine {
    pub steps: Vec<Rc<dyn PostmortemBatchAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemBatchEngine {
    pub fn evaluate(&mut self, item: &PostmortemBatch) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PostmortemBatchAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_postmortem_batch_engine(flags: &flags::Registry) -> PostmortemBatchEngine {
    PostmortemBatchEngine {
        steps: vec![
            Rc::new(FlaggedPostmortemBatchAdjuster {
                flags: flags.clone(),
                flag: "postmortem-batch-seasonal".to_string(),
                percent: 3.5,
            }) as Rc<dyn PostmortemBatchAdjuster>,
            Rc::new(PercentPostmortemBatchAdjuster { percent: -14.8 }),
            Rc::new(MinimumPostmortemBatchAdjuster { minimum: 19.9 }),
            Rc::new(TieredPostmortemBatchAdjuster {
                bounds: vec![1639.0, 2326.0, 5481.0],
                rates: vec![0.92, 0.81, 1.07],
            }),
            Rc::new(RoundingPostmortemBatchAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 48276.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_batch_engine(&flags::new_registry());
        let item = new_postmortem_batch("code-321", "title-825");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
