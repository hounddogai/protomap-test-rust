use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled migration segment calculation.
pub trait ScheduledMigrationSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledMigrationSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledMigrationSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledMigrationSegmentAdjuster for TieredScheduledMigrationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledMigrationSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledMigrationSegmentAdjuster {
    pub percent: f64,
}

impl ScheduledMigrationSegmentAdjuster for PercentScheduledMigrationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledMigrationSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledMigrationSegmentAdjuster {
    pub step: f64,
}

impl ScheduledMigrationSegmentAdjuster for RoundingScheduledMigrationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledMigrationSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled migration segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledMigrationSegmentEngine {
    pub steps: Vec<Rc<dyn ScheduledMigrationSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledMigrationSegmentEngine {
    pub fn price(&mut self, item: &ScheduledMigrationSegment) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledMigrationSegmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledMigrationSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_migration_segment_engine() -> ScheduledMigrationSegmentEngine {
    ScheduledMigrationSegmentEngine {
        steps: vec![
            Rc::new(TieredScheduledMigrationSegmentAdjuster {
                bounds: vec![2390.0, 7715.0, 8868.0, 9523.0],
                rates: vec![1.16, 1.11, 1.09, 1.1],
            }) as Rc<dyn ScheduledMigrationSegmentAdjuster>,
            Rc::new(PercentScheduledMigrationSegmentAdjuster { percent: 20.4 }),
            Rc::new(RoundingScheduledMigrationSegmentAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 58850.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_migration_segment_engine();
        let item = new_scheduled_migration_segment("code-65", "name-302");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
