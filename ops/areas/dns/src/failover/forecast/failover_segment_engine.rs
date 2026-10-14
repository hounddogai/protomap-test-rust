use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the failover segment calculation.
pub trait FailoverSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFailoverSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverSegmentAdjuster for LabelFailoverSegmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FailoverSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFailoverSegmentAdjuster {
    pub step: f64,
}

impl FailoverSegmentAdjuster for RoundingFailoverSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FailoverSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverSegmentAdjuster for FlaggedFailoverSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverSegmentAdjuster {
    pub cap: f64,
}

impl FailoverSegmentAdjuster for CapFailoverSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFailoverSegmentAdjuster {
    pub minimum: f64,
}

impl FailoverSegmentAdjuster for MinimumFailoverSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FailoverSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the failover segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverSegmentEngine {
    pub steps: Vec<Rc<dyn FailoverSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverSegmentEngine {
    pub fn run(&mut self, item: &FailoverSegment) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverSegmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_failover_segment_engine(flags: &flags::Registry) -> FailoverSegmentEngine {
    FailoverSegmentEngine {
        steps: vec![
            Rc::new(LabelFailoverSegmentAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.71), ("central".to_string(), 1.07)]),
            }) as Rc<dyn FailoverSegmentAdjuster>,
            Rc::new(RoundingFailoverSegmentAdjuster { step: 0.25 }),
            Rc::new(FlaggedFailoverSegmentAdjuster {
                flags: flags.clone(),
                flag: "failover-segment-rolling".to_string(),
                percent: -8.2,
            }),
            Rc::new(CapFailoverSegmentAdjuster { cap: 17432.0 }),
            Rc::new(MinimumFailoverSegmentAdjuster { minimum: 47.6 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 75081.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_segment_engine(&flags::new_registry());
        let item = new_failover_segment("code-967", "label-304");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
