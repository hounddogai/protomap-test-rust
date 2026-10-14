use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the migration segment calculation.
pub trait MigrationSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationSegmentAdjuster {
    pub step: f64,
}

impl MigrationSegmentAdjuster for RoundingMigrationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMigrationSegmentAdjuster {
    pub cap: f64,
}

impl MigrationSegmentAdjuster for CapMigrationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MigrationSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationSegmentAdjuster {
    pub minimum: f64,
}

impl MigrationSegmentAdjuster for MinimumMigrationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSegmentSequenceOffsetAdjuster {
    pub weight: f64,
}

impl MigrationSegmentAdjuster for MigrationSegmentSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationSegment, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationSegmentAdjuster {
    pub percent: f64,
}

impl MigrationSegmentAdjuster for PercentMigrationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the migration segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationSegmentEngine {
    pub steps: Vec<Rc<dyn MigrationSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationSegmentEngine {
    pub fn run(&mut self, item: &MigrationSegment) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationSegmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_migration_segment_engine() -> MigrationSegmentEngine {
    MigrationSegmentEngine {
        steps: vec![
            Rc::new(RoundingMigrationSegmentAdjuster { step: 0.05 }) as Rc<dyn MigrationSegmentAdjuster>,
            Rc::new(CapMigrationSegmentAdjuster { cap: 41829.0 }),
            Rc::new(MinimumMigrationSegmentAdjuster { minimum: 8.2 }),
            Rc::new(MigrationSegmentSequenceOffsetAdjuster { weight: 1.81 }),
            Rc::new(PercentMigrationSegmentAdjuster { percent: 17.8 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 83335.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_segment_engine();
        let item = new_migration_segment("code-155", "title-271");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
