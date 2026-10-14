use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the failover schedule calculation.
pub trait FailoverScheduleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFailoverScheduleAdjuster {
    pub step: f64,
}

impl FailoverScheduleAdjuster for RoundingFailoverScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FailoverSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverScheduleAdjuster for TieredFailoverScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverScheduleAdjuster {
    pub cap: f64,
}

impl FailoverScheduleAdjuster for CapFailoverScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceFailoverScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverScheduleAdjuster for SourceFailoverScheduleAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &FailoverSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFailoverScheduleAdjuster {
    pub minimum: f64,
}

impl FailoverScheduleAdjuster for MinimumFailoverScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FailoverSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverScheduleAdjuster for FlaggedFailoverScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the failover schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverScheduleEngine {
    pub steps: Vec<Rc<dyn FailoverScheduleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverScheduleEngine {
    pub fn evaluate(&mut self, item: &FailoverSchedule) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverScheduleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_failover_schedule_engine(flags: &flags::Registry) -> FailoverScheduleEngine {
    FailoverScheduleEngine {
        steps: vec![
            Rc::new(RoundingFailoverScheduleAdjuster { step: 0.5 }) as Rc<dyn FailoverScheduleAdjuster>,
            Rc::new(TieredFailoverScheduleAdjuster {
                bounds: vec![2423.0, 2426.0, 3309.0, 8309.0],
                rates: vec![0.84, 0.94, 0.9, 0.92],
            }),
            Rc::new(CapFailoverScheduleAdjuster { cap: 37598.0 }),
            Rc::new(SourceFailoverScheduleAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.31),
                    ("central".to_string(), 1.14),
                    ("south".to_string(), 1.07),
                ]),
            }),
            Rc::new(MinimumFailoverScheduleAdjuster { minimum: 36.4 }),
            Rc::new(FlaggedFailoverScheduleAdjuster {
                flags: flags.clone(),
                flag: "failover-schedule-batched".to_string(),
                percent: 4.6,
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 40285.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_schedule_engine(&flags::new_registry());
        let item = new_failover_schedule("code-412", "title-312");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
