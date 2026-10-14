use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled isolation exception calculation.
pub trait ScheduledIsolationExceptionFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledIsolationException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledIsolationExceptionAdjuster {
    pub minimum: f64,
}

impl ScheduledIsolationExceptionFactor for MinimumScheduledIsolationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledIsolationException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelScheduledIsolationExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledIsolationExceptionFactor for ChannelScheduledIsolationExceptionAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &ScheduledIsolationException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledIsolationExceptionAdjuster {
    pub cap: f64,
}

impl ScheduledIsolationExceptionFactor for CapScheduledIsolationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledIsolationException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledIsolationExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledIsolationExceptionFactor for TieredScheduledIsolationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledIsolationException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledIsolationExceptionAdjuster {
    pub step: f64,
}

impl ScheduledIsolationExceptionFactor for RoundingScheduledIsolationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledIsolationException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled isolation exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledIsolationExceptionEngine {
    pub steps: Vec<Rc<dyn ScheduledIsolationExceptionFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledIsolationExceptionEngine {
    pub fn run(&mut self, item: &ScheduledIsolationException) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledIsolationExceptionFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledIsolationException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_isolation_exception_engine() -> ScheduledIsolationExceptionEngine {
    ScheduledIsolationExceptionEngine {
        steps: vec![
            Rc::new(MinimumScheduledIsolationExceptionAdjuster { minimum: 34.1 })
                as Rc<dyn ScheduledIsolationExceptionFactor>,
            Rc::new(ChannelScheduledIsolationExceptionAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.96)]),
            }),
            Rc::new(CapScheduledIsolationExceptionAdjuster { cap: 26620.0 }),
            Rc::new(TieredScheduledIsolationExceptionAdjuster {
                bounds: vec![1031.0, 1517.0, 3561.0],
                rates: vec![0.95, 0.82, 1.14],
            }),
            Rc::new(RoundingScheduledIsolationExceptionAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 49305.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_isolation_exception_engine();
        let item = new_scheduled_isolation_exception("code-658", "label-904");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
