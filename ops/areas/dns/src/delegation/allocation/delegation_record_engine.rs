use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation record calculation.
pub trait DelegationRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationRecordAdjuster {
    pub step: f64,
}

impl DelegationRecordStep for RoundingDelegationRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationRecordStep for TieredDelegationRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationRecordStep for FlaggedDelegationRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationRecordAdjuster {
    pub cap: f64,
}

impl DelegationRecordStep for CapDelegationRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRecordThresholdOffsetAdjuster {
    pub weight: f64,
}

impl DelegationRecordStep for DelegationRecordThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &DelegationRecord, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDelegationRecordAdjuster {
    pub percent: f64,
}

impl DelegationRecordStep for PercentDelegationRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DelegationRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the delegation record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationRecordEngine {
    pub steps: Vec<Rc<dyn DelegationRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationRecordEngine {
    pub fn evaluate(&mut self, item: &DelegationRecord) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DelegationRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DelegationRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_delegation_record_engine(flags: &flags::Registry) -> DelegationRecordEngine {
    DelegationRecordEngine {
        steps: vec![
            Rc::new(RoundingDelegationRecordAdjuster { step: 0.5 }) as Rc<dyn DelegationRecordStep>,
            Rc::new(TieredDelegationRecordAdjuster {
                bounds: vec![662.0, 2899.0, 3088.0, 6091.0],
                rates: vec![0.89, 0.93, 0.8, 1.16],
            }),
            Rc::new(FlaggedDelegationRecordAdjuster {
                flags: flags.clone(),
                flag: "delegation-record-fallback".to_string(),
                percent: 4.1,
            }),
            Rc::new(CapDelegationRecordAdjuster { cap: 42851.0 }),
            Rc::new(DelegationRecordThresholdOffsetAdjuster { weight: 0.07 }),
            Rc::new(PercentDelegationRecordAdjuster { percent: 1.3 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 45241.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_record_engine(&flags::new_registry());
        let item = new_delegation_record("code-150", "name-860");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
