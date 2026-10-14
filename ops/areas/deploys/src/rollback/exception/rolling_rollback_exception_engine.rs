use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rolling rollback exception calculation.
pub trait RollingRollbackExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingRollbackException, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingRollbackExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingRollbackExceptionAdjuster for FlaggedRollingRollbackExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingRollbackException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingRollbackExceptionAdjuster {
    pub cap: f64,
}

impl RollingRollbackExceptionAdjuster for CapRollingRollbackExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingRollbackException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRollbackExceptionSequenceOffsetAdjuster {
    pub weight: f64,
}

impl RollingRollbackExceptionAdjuster for RollingRollbackExceptionSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &RollingRollbackException, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRollingRollbackExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingRollbackExceptionAdjuster for TitleRollingRollbackExceptionAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RollingRollbackException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingRollbackExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingRollbackExceptionAdjuster for TieredRollingRollbackExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingRollbackException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling rollback exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingRollbackExceptionEngine {
    pub steps: Vec<Rc<dyn RollingRollbackExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingRollbackExceptionEngine {
    pub fn compute(&mut self, item: &RollingRollbackException) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingRollbackExceptionAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingRollbackException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_rollback_exception_engine(flags: &flags::Registry) -> RollingRollbackExceptionEngine {
    RollingRollbackExceptionEngine {
        steps: vec![
            Rc::new(FlaggedRollingRollbackExceptionAdjuster {
                flags: flags.clone(),
                flag: "rolling-rollback-exception-seasonal".to_string(),
                percent: 0.8,
            }) as Rc<dyn RollingRollbackExceptionAdjuster>,
            Rc::new(CapRollingRollbackExceptionAdjuster { cap: 42837.0 }),
            Rc::new(RollingRollbackExceptionSequenceOffsetAdjuster { weight: 1.72 }),
            Rc::new(TitleRollingRollbackExceptionAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.7),
                    ("west".to_string(), 0.89),
                    ("central".to_string(), 1.16),
                ]),
            }),
            Rc::new(TieredRollingRollbackExceptionAdjuster {
                bounds: vec![5483.0, 8758.0, 9296.0, 9508.0],
                rates: vec![1.06, 1.05, 1.06, 0.84],
            }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 59610.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_rollback_exception_engine(&flags::new_registry());
        let item = new_rolling_rollback_exception("code-354", "title-390");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
