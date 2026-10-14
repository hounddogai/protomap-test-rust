use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered runbook record calculation.
pub trait TieredRunbookRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredRunbookRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredRunbookRecordAdjuster {
    pub step: f64,
}

impl TieredRunbookRecordModifier for RoundingTieredRunbookRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredRunbookRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredRunbookRecordAdjuster {
    pub percent: f64,
}

impl TieredRunbookRecordModifier for PercentTieredRunbookRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredRunbookRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredRunbookRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredRunbookRecordModifier for FlaggedTieredRunbookRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredRunbookRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered runbook record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredRunbookRecordEngine {
    pub steps: Vec<Rc<dyn TieredRunbookRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredRunbookRecordEngine {
    pub fn run(&mut self, item: &TieredRunbookRecord) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredRunbookRecordModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredRunbookRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_runbook_record_engine(flags: &flags::Registry) -> TieredRunbookRecordEngine {
    TieredRunbookRecordEngine {
        steps: vec![
            Rc::new(RoundingTieredRunbookRecordAdjuster { step: 0.5 }) as Rc<dyn TieredRunbookRecordModifier>,
            Rc::new(PercentTieredRunbookRecordAdjuster { percent: 23.3 }),
            Rc::new(FlaggedTieredRunbookRecordAdjuster {
                flags: flags.clone(),
                flag: "tiered-runbook-record-primary".to_string(),
                percent: -9.4,
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 32153.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_runbook_record_engine(&flags::new_registry());
        let item = new_tiered_runbook_record("code-644", "title-960");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
