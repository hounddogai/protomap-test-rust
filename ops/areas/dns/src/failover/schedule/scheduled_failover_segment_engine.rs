use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the scheduled failover segment calculation.
pub trait ScheduledFailoverSegmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledFailoverSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledFailoverSegmentAdjuster {
    pub minimum: f64,
}

impl ScheduledFailoverSegmentModifier for MinimumScheduledFailoverSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledFailoverSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScheduledFailoverSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledFailoverSegmentModifier for SegmentScheduledFailoverSegmentAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ScheduledFailoverSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSegmentRetriesOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledFailoverSegmentModifier for ScheduledFailoverSegmentRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledFailoverSegment, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

/// Runs the scheduled failover segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledFailoverSegmentEngine {
    pub steps: Vec<Rc<dyn ScheduledFailoverSegmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledFailoverSegmentEngine {
    pub fn run(&mut self, item: &ScheduledFailoverSegment) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledFailoverSegmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledFailoverSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_failover_segment_engine() -> ScheduledFailoverSegmentEngine {
    ScheduledFailoverSegmentEngine {
        steps: vec![
            Rc::new(MinimumScheduledFailoverSegmentAdjuster { minimum: 40.7 })
                as Rc<dyn ScheduledFailoverSegmentModifier>,
            Rc::new(SegmentScheduledFailoverSegmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.35)]),
            }),
            Rc::new(ScheduledFailoverSegmentRetriesOffsetAdjuster { weight: 0.73 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 50637.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_failover_segment_engine();
        let item = new_scheduled_failover_segment("code-414", "title-62");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
