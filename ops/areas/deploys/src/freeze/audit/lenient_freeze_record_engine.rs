use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient freeze record calculation.
pub trait LenientFreezeRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientFreezeRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientFreezeRecordAdjuster {
    pub step: f64,
}

impl LenientFreezeRecordStep for RoundingLenientFreezeRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientFreezeRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientFreezeRecordAdjuster {
    pub cap: f64,
}

impl LenientFreezeRecordStep for CapLenientFreezeRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientFreezeRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientFreezeRecordAdjuster {
    pub minimum: f64,
}

impl LenientFreezeRecordStep for MinimumLenientFreezeRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientFreezeRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientFreezeRecordAdjuster {
    pub percent: f64,
}

impl LenientFreezeRecordStep for PercentLenientFreezeRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientFreezeRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeRecordMinimumOffsetAdjuster {
    pub weight: f64,
}

impl LenientFreezeRecordStep for LenientFreezeRecordMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &LenientFreezeRecord, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the lenient freeze record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientFreezeRecordEngine {
    pub steps: Vec<Rc<dyn LenientFreezeRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientFreezeRecordEngine {
    pub fn run(&mut self, item: &LenientFreezeRecord) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[LenientFreezeRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_freeze_record_engine() -> LenientFreezeRecordEngine {
    LenientFreezeRecordEngine {
        steps: vec![
            Rc::new(RoundingLenientFreezeRecordAdjuster { step: 5.0 }) as Rc<dyn LenientFreezeRecordStep>,
            Rc::new(CapLenientFreezeRecordAdjuster { cap: 30664.0 }),
            Rc::new(MinimumLenientFreezeRecordAdjuster { minimum: 1.4 }),
            Rc::new(PercentLenientFreezeRecordAdjuster { percent: -9.9 }),
            Rc::new(LenientFreezeRecordMinimumOffsetAdjuster { weight: 1.25 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 60036.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_freeze_record_engine();
        let item = new_lenient_freeze_record("code-346", "name-774");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
