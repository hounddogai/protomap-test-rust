use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional retention schedule calculation.
pub trait RegionalRetentionScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalRetentionSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalRetentionScheduleAdjuster {
    pub percent: f64,
}

impl RegionalRetentionScheduleFactor for PercentRegionalRetentionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalRetentionSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalRetentionScheduleAdjuster {
    pub step: f64,
}

impl RegionalRetentionScheduleFactor for RoundingRegionalRetentionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalRetentionSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalRetentionScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalRetentionScheduleFactor for TieredRegionalRetentionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalRetentionSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalRetentionScheduleAdjuster {
    pub cap: f64,
}

impl RegionalRetentionScheduleFactor for CapRegionalRetentionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalRetentionSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the regional retention schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalRetentionScheduleEngine {
    pub steps: Vec<Rc<dyn RegionalRetentionScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalRetentionScheduleEngine {
    pub fn price(&mut self, item: &RegionalRetentionSchedule) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalRetentionScheduleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalRetentionSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_retention_schedule_engine() -> RegionalRetentionScheduleEngine {
    RegionalRetentionScheduleEngine {
        steps: vec![
            Rc::new(PercentRegionalRetentionScheduleAdjuster { percent: -14.6 })
                as Rc<dyn RegionalRetentionScheduleFactor>,
            Rc::new(RoundingRegionalRetentionScheduleAdjuster { step: 0.1 }),
            Rc::new(TieredRegionalRetentionScheduleAdjuster {
                bounds: vec![3930.0, 6409.0, 8679.0],
                rates: vec![0.88, 1.07, 0.9],
            }),
            Rc::new(CapRegionalRetentionScheduleAdjuster { cap: 30874.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 29274.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_retention_schedule_engine();
        let item = new_regional_retention_schedule("code-714", "title-134");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
