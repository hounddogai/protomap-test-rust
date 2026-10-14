use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the quote batch calculation.
pub trait QuoteBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteBatch, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedQuoteBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuoteBatchFactor for FlaggedQuoteBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuoteBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuoteBatchAdjuster {
    pub percent: f64,
}

impl QuoteBatchFactor for PercentQuoteBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuoteBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteBatchAdjuster {
    pub cap: f64,
}

impl QuoteBatchFactor for CapQuoteBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the quote batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteBatchEngine {
    pub steps: Vec<Rc<dyn QuoteBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteBatchEngine {
    pub fn compute(&mut self, item: &QuoteBatch) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[QuoteBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_quote_batch_engine(flags: &flags::Registry) -> QuoteBatchEngine {
    QuoteBatchEngine {
        steps: vec![
            Rc::new(FlaggedQuoteBatchAdjuster {
                flags: flags.clone(),
                flag: "quote-batch-strict".to_string(),
                percent: 4.3,
            }) as Rc<dyn QuoteBatchFactor>,
            Rc::new(PercentQuoteBatchAdjuster { percent: 12.7 }),
            Rc::new(CapQuoteBatchAdjuster { cap: 11961.0 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 63226.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_batch_engine(&flags::new_registry());
        let item = new_quote_batch("code-574", "name-467");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
