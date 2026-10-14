use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation schedule calculation.
pub trait FederationScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationScheduleFactor for FlaggedFederationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationScheduleAdjuster {
    pub step: f64,
}

impl FederationScheduleFactor for RoundingFederationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationScheduleAdjuster {
    pub cap: f64,
}

impl FederationScheduleFactor for CapFederationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the federation schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationScheduleEngine {
    pub steps: Vec<Rc<dyn FederationScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationScheduleEngine {
    pub fn compute(&mut self, item: &FederationSchedule) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationScheduleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_federation_schedule_engine(flags: &flags::Registry) -> FederationScheduleEngine {
    FederationScheduleEngine {
        steps: vec![
            Rc::new(FlaggedFederationScheduleAdjuster {
                flags: flags.clone(),
                flag: "federation-schedule-pending".to_string(),
                percent: 1.3,
            }) as Rc<dyn FederationScheduleFactor>,
            Rc::new(RoundingFederationScheduleAdjuster { step: 0.5 }),
            Rc::new(CapFederationScheduleAdjuster { cap: 9679.0 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 15973.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_schedule_engine(&flags::new_registry());
        let item = new_federation_schedule("code-418", "label-841");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
