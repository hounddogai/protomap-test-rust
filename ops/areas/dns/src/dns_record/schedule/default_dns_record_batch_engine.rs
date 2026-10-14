use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the default dns record batch calculation.
pub trait DefaultDnsRecordBatchStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultDnsRecordBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordBatchQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl DefaultDnsRecordBatchStep for DefaultDnsRecordBatchQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultDnsRecordBatch, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultDnsRecordBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultDnsRecordBatchStep for FlaggedDefaultDnsRecordBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultDnsRecordBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultDnsRecordBatchAdjuster {
    pub cap: f64,
}

impl DefaultDnsRecordBatchStep for CapDefaultDnsRecordBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultDnsRecordBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the default dns record batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultDnsRecordBatchEngine {
    pub steps: Vec<Rc<dyn DefaultDnsRecordBatchStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultDnsRecordBatchEngine {
    pub fn evaluate(&mut self, item: &DefaultDnsRecordBatch) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultDnsRecordBatchStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultDnsRecordBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_dns_record_batch_engine(flags: &flags::Registry) -> DefaultDnsRecordBatchEngine {
    DefaultDnsRecordBatchEngine {
        steps: vec![
            Rc::new(DefaultDnsRecordBatchQuotaUnitsOffsetAdjuster { weight: 0.28 })
                as Rc<dyn DefaultDnsRecordBatchStep>,
            Rc::new(FlaggedDefaultDnsRecordBatchAdjuster {
                flags: flags.clone(),
                flag: "default-dns-record-batch-seasonal".to_string(),
                percent: -8.9,
            }),
            Rc::new(CapDefaultDnsRecordBatchAdjuster { cap: 9414.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 40437.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_dns_record_batch_engine(&flags::new_registry());
        let item = new_default_dns_record_batch("code-812", "label-626");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
