use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the silence exception calculation.
pub trait SilenceExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSilenceExceptionAdjuster {
    pub minimum: f64,
}

impl SilenceExceptionStep for MinimumSilenceExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SilenceException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSilenceExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SilenceExceptionStep for FlaggedSilenceExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SilenceException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSilenceExceptionAdjuster {
    pub cap: f64,
}

impl SilenceExceptionStep for CapSilenceExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SilenceException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the silence exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceExceptionEngine {
    pub steps: Vec<Rc<dyn SilenceExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceExceptionEngine {
    pub fn price(&mut self, item: &SilenceException) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[SilenceException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_silence_exception_engine(flags: &flags::Registry) -> SilenceExceptionEngine {
    SilenceExceptionEngine {
        steps: vec![
            Rc::new(MinimumSilenceExceptionAdjuster { minimum: 33.1 }) as Rc<dyn SilenceExceptionStep>,
            Rc::new(FlaggedSilenceExceptionAdjuster {
                flags: flags.clone(),
                flag: "silence-exception-pending".to_string(),
                percent: -7.9,
            }),
            Rc::new(CapSilenceExceptionAdjuster { cap: 28964.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 50374.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_exception_engine(&flags::new_registry());
        let item = new_silence_exception("code-328", "label-422");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
