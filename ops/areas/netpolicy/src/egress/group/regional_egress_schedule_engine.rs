use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional egress schedule calculation.
pub trait RegionalEgressScheduleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalEgressSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalEgressScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalEgressScheduleAdjuster for TieredRegionalEgressScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalEgressSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalEgressScheduleAdjuster {
    pub step: f64,
}

impl RegionalEgressScheduleAdjuster for RoundingRegionalEgressScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalEgressSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalEgressScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalEgressScheduleAdjuster for FlaggedRegionalEgressScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalEgressSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRegionalEgressScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalEgressScheduleAdjuster for TitleRegionalEgressScheduleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RegionalEgressSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalEgressScheduleAdjuster {
    pub cap: f64,
}

impl RegionalEgressScheduleAdjuster for CapRegionalEgressScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalEgressSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalEgressScheduleAdjuster {
    pub minimum: f64,
}

impl RegionalEgressScheduleAdjuster for MinimumRegionalEgressScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalEgressSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional egress schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalEgressScheduleEngine {
    pub steps: Vec<Rc<dyn RegionalEgressScheduleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalEgressScheduleEngine {
    pub fn run(&mut self, item: &RegionalEgressSchedule) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalEgressScheduleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalEgressSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_egress_schedule_engine(flags: &flags::Registry) -> RegionalEgressScheduleEngine {
    RegionalEgressScheduleEngine {
        steps: vec![
            Rc::new(TieredRegionalEgressScheduleAdjuster {
                bounds: vec![3843.0, 4575.0, 5146.0],
                rates: vec![1.17, 0.85, 1.17],
            }) as Rc<dyn RegionalEgressScheduleAdjuster>,
            Rc::new(RoundingRegionalEgressScheduleAdjuster { step: 0.05 }),
            Rc::new(FlaggedRegionalEgressScheduleAdjuster {
                flags: flags.clone(),
                flag: "regional-egress-schedule-scheduled".to_string(),
                percent: 2.3,
            }),
            Rc::new(TitleRegionalEgressScheduleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.75)]),
            }),
            Rc::new(CapRegionalEgressScheduleAdjuster { cap: 20797.0 }),
            Rc::new(MinimumRegionalEgressScheduleAdjuster { minimum: 19.9 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 42198.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_egress_schedule_engine(&flags::new_registry());
        let item = new_regional_egress_schedule("code-192", "title-901");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
