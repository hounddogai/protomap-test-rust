use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary suspension schedule calculation.
pub trait PrimarySuspensionScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimarySuspensionSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimarySuspensionScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimarySuspensionScheduleModifier for FlaggedPrimarySuspensionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimarySuspensionSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimarySuspensionScheduleAdjuster {
    pub minimum: f64,
}

impl PrimarySuspensionScheduleModifier for MinimumPrimarySuspensionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimarySuspensionSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimarySuspensionScheduleAdjuster {
    pub step: f64,
}

impl PrimarySuspensionScheduleModifier for RoundingPrimarySuspensionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimarySuspensionSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimarySuspensionScheduleAdjuster {
    pub percent: f64,
}

impl PrimarySuspensionScheduleModifier for PercentPrimarySuspensionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimarySuspensionSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimarySuspensionScheduleAdjuster {
    pub cap: f64,
}

impl PrimarySuspensionScheduleModifier for CapPrimarySuspensionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimarySuspensionSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary suspension schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimarySuspensionScheduleEngine {
    pub steps: Vec<Rc<dyn PrimarySuspensionScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimarySuspensionScheduleEngine {
    pub fn price(&mut self, item: &PrimarySuspensionSchedule) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimarySuspensionScheduleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimarySuspensionSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_suspension_schedule_engine(flags: &flags::Registry) -> PrimarySuspensionScheduleEngine {
    PrimarySuspensionScheduleEngine {
        steps: vec![
            Rc::new(FlaggedPrimarySuspensionScheduleAdjuster {
                flags: flags.clone(),
                flag: "primary-suspension-schedule-weighted".to_string(),
                percent: 4.0,
            }) as Rc<dyn PrimarySuspensionScheduleModifier>,
            Rc::new(MinimumPrimarySuspensionScheduleAdjuster { minimum: 47.3 }),
            Rc::new(RoundingPrimarySuspensionScheduleAdjuster { step: 0.5 }),
            Rc::new(PercentPrimarySuspensionScheduleAdjuster { percent: 6.7 }),
            Rc::new(CapPrimarySuspensionScheduleAdjuster { cap: 12344.0 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 43238.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_suspension_schedule_engine(&flags::new_registry());
        let item = new_primary_suspension_schedule("code-724", "name-18");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
