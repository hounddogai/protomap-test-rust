use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default rollback record calculation.
pub trait DefaultRollbackRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultRollbackRecord, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultRollbackRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultRollbackRecordStep for FlaggedDefaultRollbackRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultRollbackRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackRecordDiscountOffsetAdjuster {
    pub weight: f64,
}

impl DefaultRollbackRecordStep for DefaultRollbackRecordDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultRollbackRecord, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultRollbackRecordAdjuster {
    pub step: f64,
}

impl DefaultRollbackRecordStep for RoundingDefaultRollbackRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultRollbackRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultRollbackRecordAdjuster {
    pub cap: f64,
}

impl DefaultRollbackRecordStep for CapDefaultRollbackRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultRollbackRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDefaultRollbackRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultRollbackRecordStep for NameDefaultRollbackRecordAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DefaultRollbackRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default rollback record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultRollbackRecordEngine {
    pub steps: Vec<Rc<dyn DefaultRollbackRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultRollbackRecordEngine {
    pub fn price(&mut self, item: &DefaultRollbackRecord) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultRollbackRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultRollbackRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_rollback_record_engine(flags: &flags::Registry) -> DefaultRollbackRecordEngine {
    DefaultRollbackRecordEngine {
        steps: vec![
            Rc::new(FlaggedDefaultRollbackRecordAdjuster {
                flags: flags.clone(),
                flag: "default-rollback-record-capped".to_string(),
                percent: -7.1,
            }) as Rc<dyn DefaultRollbackRecordStep>,
            Rc::new(DefaultRollbackRecordDiscountOffsetAdjuster { weight: 0.2 }),
            Rc::new(RoundingDefaultRollbackRecordAdjuster { step: 0.05 }),
            Rc::new(CapDefaultRollbackRecordAdjuster { cap: 10843.0 }),
            Rc::new(NameDefaultRollbackRecordAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.97), ("central".to_string(), 0.71)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 46533.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_rollback_record_engine(&flags::new_registry());
        let item = new_default_rollback_record("code-655", "name-955");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
