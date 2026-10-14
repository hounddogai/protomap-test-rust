use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped freeze exception calculation.
pub trait CappedFreezeExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedFreezeException, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedFreezeExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedFreezeExceptionAdjuster for FlaggedCappedFreezeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedFreezeException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedFreezeExceptionAdjuster {
    pub step: f64,
}

impl CappedFreezeExceptionAdjuster for RoundingCappedFreezeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedFreezeException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFreezeExceptionCapacityOffsetAdjuster {
    pub weight: f64,
}

impl CappedFreezeExceptionAdjuster for CappedFreezeExceptionCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &CappedFreezeException, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedFreezeExceptionAdjuster {
    pub minimum: f64,
}

impl CappedFreezeExceptionAdjuster for MinimumCappedFreezeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedFreezeException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedFreezeExceptionAdjuster {
    pub percent: f64,
}

impl CappedFreezeExceptionAdjuster for PercentCappedFreezeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedFreezeException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedFreezeExceptionAdjuster {
    pub cap: f64,
}

impl CappedFreezeExceptionAdjuster for CapCappedFreezeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedFreezeException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped freeze exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedFreezeExceptionEngine {
    pub steps: Vec<Rc<dyn CappedFreezeExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedFreezeExceptionEngine {
    pub fn run(&mut self, item: &CappedFreezeException) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedFreezeExceptionAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedFreezeException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_freeze_exception_engine(flags: &flags::Registry) -> CappedFreezeExceptionEngine {
    CappedFreezeExceptionEngine {
        steps: vec![
            Rc::new(FlaggedCappedFreezeExceptionAdjuster {
                flags: flags.clone(),
                flag: "capped-freeze-exception-default".to_string(),
                percent: 8.4,
            }) as Rc<dyn CappedFreezeExceptionAdjuster>,
            Rc::new(RoundingCappedFreezeExceptionAdjuster { step: 0.5 }),
            Rc::new(CappedFreezeExceptionCapacityOffsetAdjuster { weight: 1.22 }),
            Rc::new(MinimumCappedFreezeExceptionAdjuster { minimum: 10.0 }),
            Rc::new(PercentCappedFreezeExceptionAdjuster { percent: -12.3 }),
            Rc::new(CapCappedFreezeExceptionAdjuster { cap: 39642.0 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 61341.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_freeze_exception_engine(&flags::new_registry());
        let item = new_capped_freeze_exception("code-911", "label-599");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
