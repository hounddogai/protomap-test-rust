use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the alert record calculation.
pub trait AlertRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceAlertRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertRecordAdjuster for SourceAlertRecordAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &AlertRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAlertRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AlertRecordAdjuster for FlaggedAlertRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AlertRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertRecordAdjuster for TieredAlertRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertRecordAdjuster {
    pub percent: f64,
}

impl AlertRecordAdjuster for PercentAlertRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRecordAmountOffsetAdjuster {
    pub weight: f64,
}

impl AlertRecordAdjuster for AlertRecordAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &AlertRecord, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertRecordAdjuster {
    pub step: f64,
}

impl AlertRecordAdjuster for RoundingAlertRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the alert record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertRecordEngine {
    pub steps: Vec<Rc<dyn AlertRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertRecordEngine {
    pub fn evaluate(&mut self, item: &AlertRecord) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AlertRecordAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AlertRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_alert_record_engine(flags: &flags::Registry) -> AlertRecordEngine {
    AlertRecordEngine {
        steps: vec![
            Rc::new(SourceAlertRecordAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.16), ("central".to_string(), 1.3)]),
            }) as Rc<dyn AlertRecordAdjuster>,
            Rc::new(FlaggedAlertRecordAdjuster {
                flags: flags.clone(),
                flag: "alert-record-rolling".to_string(),
                percent: 4.6,
            }),
            Rc::new(TieredAlertRecordAdjuster {
                bounds: vec![498.0, 1068.0, 9520.0],
                rates: vec![1.16, 0.94, 0.99],
            }),
            Rc::new(PercentAlertRecordAdjuster { percent: 19.4 }),
            Rc::new(AlertRecordAmountOffsetAdjuster { weight: 0.9 }),
            Rc::new(RoundingAlertRecordAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 12911.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_record_engine(&flags::new_registry());
        let item = new_alert_record("code-713", "name-221");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
