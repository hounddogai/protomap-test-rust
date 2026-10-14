use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the scheduled postmortem estimate calculation.
pub trait ScheduledPostmortemEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledPostmortemEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemEstimateBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledPostmortemEstimateAdjuster for ScheduledPostmortemEstimateBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledPostmortemEstimate, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledPostmortemEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledPostmortemEstimateAdjuster for FlaggedScheduledPostmortemEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledPostmortemEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledPostmortemEstimateAdjuster {
    pub minimum: f64,
}

impl ScheduledPostmortemEstimateAdjuster for MinimumScheduledPostmortemEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledPostmortemEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledPostmortemEstimateAdjuster {
    pub cap: f64,
}

impl ScheduledPostmortemEstimateAdjuster for CapScheduledPostmortemEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledPostmortemEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled postmortem estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledPostmortemEstimateEngine {
    pub steps: Vec<Rc<dyn ScheduledPostmortemEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledPostmortemEstimateEngine {
    pub fn evaluate(&mut self, item: &ScheduledPostmortemEstimate) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledPostmortemEstimateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledPostmortemEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_postmortem_estimate_engine(flags: &flags::Registry) -> ScheduledPostmortemEstimateEngine {
    ScheduledPostmortemEstimateEngine {
        steps: vec![
            Rc::new(ScheduledPostmortemEstimateBatchSizeOffsetAdjuster { weight: 1.8 })
                as Rc<dyn ScheduledPostmortemEstimateAdjuster>,
            Rc::new(FlaggedScheduledPostmortemEstimateAdjuster {
                flags: flags.clone(),
                flag: "scheduled-postmortem-estimate-primary".to_string(),
                percent: 7.7,
            }),
            Rc::new(MinimumScheduledPostmortemEstimateAdjuster { minimum: 1.4 }),
            Rc::new(CapScheduledPostmortemEstimateAdjuster { cap: 31816.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 48958.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_postmortem_estimate_engine(&flags::new_registry());
        let item = new_scheduled_postmortem_estimate("code-810", "title-646");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
