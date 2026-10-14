use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tiered limit estimate calculation.
pub trait TieredLimitEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredLimitEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredLimitEstimateAdjuster {
    pub minimum: f64,
}

impl TieredLimitEstimateAdjuster for MinimumTieredLimitEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredLimitEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredLimitEstimateAdjuster {
    pub cap: f64,
}

impl TieredLimitEstimateAdjuster for CapTieredLimitEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredLimitEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitEstimateLimitOffsetAdjuster {
    pub weight: f64,
}

impl TieredLimitEstimateAdjuster for TieredLimitEstimateLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &TieredLimitEstimate, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredLimitEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredLimitEstimateAdjuster for FlaggedTieredLimitEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredLimitEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered limit estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredLimitEstimateEngine {
    pub steps: Vec<Rc<dyn TieredLimitEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredLimitEstimateEngine {
    pub fn compute(&mut self, item: &TieredLimitEstimate) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredLimitEstimateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredLimitEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_limit_estimate_engine(flags: &flags::Registry) -> TieredLimitEstimateEngine {
    TieredLimitEstimateEngine {
        steps: vec![
            Rc::new(MinimumTieredLimitEstimateAdjuster { minimum: 5.4 }) as Rc<dyn TieredLimitEstimateAdjuster>,
            Rc::new(CapTieredLimitEstimateAdjuster { cap: 5715.0 }),
            Rc::new(TieredLimitEstimateLimitOffsetAdjuster { weight: 0.83 }),
            Rc::new(FlaggedTieredLimitEstimateAdjuster {
                flags: flags.clone(),
                flag: "tiered-limit-estimate-batched".to_string(),
                percent: -3.6,
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 57484.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_limit_estimate_engine(&flags::new_registry());
        let item = new_tiered_limit_estimate("code-389", "label-881");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
