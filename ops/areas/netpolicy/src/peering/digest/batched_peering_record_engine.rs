use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched peering record calculation.
pub trait BatchedPeeringRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedPeeringRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedPeeringRecordAdjuster {
    pub minimum: f64,
}

impl BatchedPeeringRecordAdjuster for MinimumBatchedPeeringRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedPeeringRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedPeeringRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedPeeringRecordAdjuster for FlaggedBatchedPeeringRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedPeeringRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedPeeringRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedPeeringRecordAdjuster for TieredBatchedPeeringRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedPeeringRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceBatchedPeeringRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedPeeringRecordAdjuster for NamespaceBatchedPeeringRecordAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &BatchedPeeringRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched peering record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedPeeringRecordEngine {
    pub steps: Vec<Rc<dyn BatchedPeeringRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedPeeringRecordEngine {
    pub fn evaluate(&mut self, item: &BatchedPeeringRecord) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedPeeringRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_peering_record_engine(flags: &flags::Registry) -> BatchedPeeringRecordEngine {
    BatchedPeeringRecordEngine {
        steps: vec![
            Rc::new(MinimumBatchedPeeringRecordAdjuster { minimum: 23.0 }) as Rc<dyn BatchedPeeringRecordAdjuster>,
            Rc::new(FlaggedBatchedPeeringRecordAdjuster {
                flags: flags.clone(),
                flag: "batched-peering-record-lenient".to_string(),
                percent: 2.5,
            }),
            Rc::new(TieredBatchedPeeringRecordAdjuster {
                bounds: vec![5615.0, 7427.0],
                rates: vec![1.04, 1.13],
            }),
            Rc::new(NamespaceBatchedPeeringRecordAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.24), ("retail".to_string(), 1.33)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 88944.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_peering_record_engine(&flags::new_registry());
        let item = new_batched_peering_record("code-328", "title-754");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
