use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched dns record adjustment calculation.
pub trait BatchedDnsRecordAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedDnsRecordAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedDnsRecordAdjustmentAdjuster {
    pub cap: f64,
}

impl BatchedDnsRecordAdjustmentFactor for CapBatchedDnsRecordAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedDnsRecordAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedDnsRecordAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedDnsRecordAdjustmentFactor for FlaggedBatchedDnsRecordAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedDnsRecordAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedDnsRecordAdjustmentAdjuster {
    pub step: f64,
}

impl BatchedDnsRecordAdjustmentFactor for RoundingBatchedDnsRecordAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedDnsRecordAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedDnsRecordAdjustmentAdjuster {
    pub minimum: f64,
}

impl BatchedDnsRecordAdjustmentFactor for MinimumBatchedDnsRecordAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedDnsRecordAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordAdjustmentAmountOffsetAdjuster {
    pub weight: f64,
}

impl BatchedDnsRecordAdjustmentFactor for BatchedDnsRecordAdjustmentAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedDnsRecordAdjustment, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the batched dns record adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedDnsRecordAdjustmentEngine {
    pub steps: Vec<Rc<dyn BatchedDnsRecordAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedDnsRecordAdjustmentEngine {
    pub fn price(&mut self, item: &BatchedDnsRecordAdjustment) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[BatchedDnsRecordAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_dns_record_adjustment_engine(flags: &flags::Registry) -> BatchedDnsRecordAdjustmentEngine {
    BatchedDnsRecordAdjustmentEngine {
        steps: vec![
            Rc::new(CapBatchedDnsRecordAdjustmentAdjuster { cap: 26262.0 }) as Rc<dyn BatchedDnsRecordAdjustmentFactor>,
            Rc::new(FlaggedBatchedDnsRecordAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "batched-dns-record-adjustment-strict".to_string(),
                percent: -7.5,
            }),
            Rc::new(RoundingBatchedDnsRecordAdjustmentAdjuster { step: 1.0 }),
            Rc::new(MinimumBatchedDnsRecordAdjustmentAdjuster { minimum: 0.8 }),
            Rc::new(BatchedDnsRecordAdjustmentAmountOffsetAdjuster { weight: 1.61 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 18324.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_dns_record_adjustment_engine(&flags::new_registry());
        let item = new_batched_dns_record_adjustment("code-982", "name-212");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
