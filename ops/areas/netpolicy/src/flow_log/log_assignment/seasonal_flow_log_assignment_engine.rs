use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal flow log assignment calculation.
pub trait SeasonalFlowLogAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalFlowLogAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalFlowLogAssignmentAdjuster {
    pub minimum: f64,
}

impl SeasonalFlowLogAssignmentFactor for MinimumSeasonalFlowLogAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalFlowLogAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalFlowLogAssignmentAdjuster {
    pub step: f64,
}

impl SeasonalFlowLogAssignmentFactor for RoundingSeasonalFlowLogAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalFlowLogAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalFlowLogAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalFlowLogAssignmentFactor for FlaggedSeasonalFlowLogAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalFlowLogAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalFlowLogAssignmentAdjuster {
    pub cap: f64,
}

impl SeasonalFlowLogAssignmentFactor for CapSeasonalFlowLogAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalFlowLogAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionSeasonalFlowLogAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalFlowLogAssignmentFactor for RegionSeasonalFlowLogAssignmentAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &SeasonalFlowLogAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the seasonal flow log assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalFlowLogAssignmentEngine {
    pub steps: Vec<Rc<dyn SeasonalFlowLogAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalFlowLogAssignmentEngine {
    pub fn price(&mut self, item: &SeasonalFlowLogAssignment) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalFlowLogAssignmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalFlowLogAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_flow_log_assignment_engine(flags: &flags::Registry) -> SeasonalFlowLogAssignmentEngine {
    SeasonalFlowLogAssignmentEngine {
        steps: vec![
            Rc::new(MinimumSeasonalFlowLogAssignmentAdjuster { minimum: 13.1 })
                as Rc<dyn SeasonalFlowLogAssignmentFactor>,
            Rc::new(RoundingSeasonalFlowLogAssignmentAdjuster { step: 0.1 }),
            Rc::new(FlaggedSeasonalFlowLogAssignmentAdjuster {
                flags: flags.clone(),
                flag: "seasonal-flow-log-assignment-tiered".to_string(),
                percent: 8.0,
            }),
            Rc::new(CapSeasonalFlowLogAssignmentAdjuster { cap: 2154.0 }),
            Rc::new(RegionSeasonalFlowLogAssignmentAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.18),
                    ("east".to_string(), 1.39),
                    ("retail".to_string(), 1.36),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 83275.1,
    }
}
