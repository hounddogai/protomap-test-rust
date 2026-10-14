use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional failover schedule calculation.
pub trait RegionalFailoverScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalFailoverSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalFailoverScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalFailoverScheduleModifier for TieredRegionalFailoverScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalFailoverSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalFailoverScheduleAdjuster {
    pub minimum: f64,
}

impl RegionalFailoverScheduleModifier for MinimumRegionalFailoverScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalFailoverSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalFailoverScheduleAdjuster {
    pub step: f64,
}

impl RegionalFailoverScheduleModifier for RoundingRegionalFailoverScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalFailoverSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalFailoverScheduleAdjuster {
    pub cap: f64,
}

impl RegionalFailoverScheduleModifier for CapRegionalFailoverScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalFailoverSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the regional failover schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalFailoverScheduleEngine {
    pub steps: Vec<Rc<dyn RegionalFailoverScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalFailoverScheduleEngine {
    pub fn run(&mut self, item: &RegionalFailoverSchedule) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalFailoverScheduleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalFailoverSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_failover_schedule_engine() -> RegionalFailoverScheduleEngine {
    RegionalFailoverScheduleEngine {
        steps: vec![
            Rc::new(TieredRegionalFailoverScheduleAdjuster {
                bounds: vec![450.0, 1774.0, 6684.0],
                rates: vec![0.94, 0.87, 1.14],
            }) as Rc<dyn RegionalFailoverScheduleModifier>,
            Rc::new(MinimumRegionalFailoverScheduleAdjuster { minimum: 8.2 }),
            Rc::new(RoundingRegionalFailoverScheduleAdjuster { step: 0.5 }),
            Rc::new(CapRegionalFailoverScheduleAdjuster { cap: 26689.0 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 65803.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_failover_schedule_engine();
        let item = new_regional_failover_schedule("code-615", "label-596");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
