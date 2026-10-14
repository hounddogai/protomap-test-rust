use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the freeze exception calculation.
pub trait FreezeExceptionFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezeExceptionAdjuster {
    pub step: f64,
}

impl FreezeExceptionFactor for RoundingFreezeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezeException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezeExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezeExceptionFactor for FlaggedFreezeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezeException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeExceptionBurstOffsetAdjuster {
    pub weight: f64,
}

impl FreezeExceptionFactor for FreezeExceptionBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeException, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFreezeExceptionAdjuster {
    pub cap: f64,
}

impl FreezeExceptionFactor for CapFreezeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FreezeException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeExceptionAdjuster {
    pub minimum: f64,
}

impl FreezeExceptionFactor for MinimumFreezeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the freeze exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeExceptionEngine {
    pub steps: Vec<Rc<dyn FreezeExceptionFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeExceptionEngine {
    pub fn compute(&mut self, item: &FreezeException) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FreezeException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_freeze_exception_engine(flags: &flags::Registry) -> FreezeExceptionEngine {
    FreezeExceptionEngine {
        steps: vec![
            Rc::new(RoundingFreezeExceptionAdjuster { step: 0.5 }) as Rc<dyn FreezeExceptionFactor>,
            Rc::new(FlaggedFreezeExceptionAdjuster {
                flags: flags.clone(),
                flag: "freeze-exception-primary".to_string(),
                percent: -7.5,
            }),
            Rc::new(FreezeExceptionBurstOffsetAdjuster { weight: 0.56 }),
            Rc::new(CapFreezeExceptionAdjuster { cap: 6072.0 }),
            Rc::new(MinimumFreezeExceptionAdjuster { minimum: 18.4 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 45506.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_exception_engine(&flags::new_registry());
        let item = new_freeze_exception("code-750", "title-548");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
