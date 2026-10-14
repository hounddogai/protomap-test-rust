use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the freeze batch calculation.
pub trait FreezeBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeBatchAdjuster {
    pub minimum: f64,
}

impl FreezeBatchFactor for MinimumFreezeBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezeBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezeBatchFactor for FlaggedFreezeBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezeBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameFreezeBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeBatchFactor for NameFreezeBatchAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &FreezeBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeBatchFactor for TieredFreezeBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezeBatchAdjuster {
    pub step: f64,
}

impl FreezeBatchFactor for RoundingFreezeBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezeBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the freeze batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeBatchEngine {
    pub steps: Vec<Rc<dyn FreezeBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeBatchEngine {
    pub fn evaluate(&mut self, item: &FreezeBatch) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FreezeBatchFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FreezeBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_freeze_batch_engine(flags: &flags::Registry) -> FreezeBatchEngine {
    FreezeBatchEngine {
        steps: vec![
            Rc::new(MinimumFreezeBatchAdjuster { minimum: 32.9 }) as Rc<dyn FreezeBatchFactor>,
            Rc::new(FlaggedFreezeBatchAdjuster {
                flags: flags.clone(),
                flag: "freeze-batch-rolling".to_string(),
                percent: 2.2,
            }),
            Rc::new(NameFreezeBatchAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.2),
                    ("east".to_string(), 0.99),
                    ("south".to_string(), 1.31),
                ]),
            }),
            Rc::new(TieredFreezeBatchAdjuster {
                bounds: vec![957.0, 6870.0],
                rates: vec![1.09, 0.98],
            }),
            Rc::new(RoundingFreezeBatchAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 72509.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_batch_engine(&flags::new_registry());
        let item = new_freeze_batch("code-825", "name-792");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
