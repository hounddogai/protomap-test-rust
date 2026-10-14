use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled probe mapping calculation.
pub trait ScheduledProbeMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledProbeMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledProbeMappingAdjuster {
    pub step: f64,
}

impl ScheduledProbeMappingStep for RoundingScheduledProbeMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledProbeMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionScheduledProbeMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledProbeMappingStep for RegionScheduledProbeMappingAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ScheduledProbeMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledProbeMappingAdjuster {
    pub minimum: f64,
}

impl ScheduledProbeMappingStep for MinimumScheduledProbeMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledProbeMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledProbeMappingAdjuster {
    pub percent: f64,
}

impl ScheduledProbeMappingStep for PercentScheduledProbeMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledProbeMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledProbeMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledProbeMappingStep for FlaggedScheduledProbeMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledProbeMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled probe mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledProbeMappingEngine {
    pub steps: Vec<Rc<dyn ScheduledProbeMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledProbeMappingEngine {
    pub fn price(&mut self, item: &ScheduledProbeMapping) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledProbeMappingStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledProbeMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_probe_mapping_engine(flags: &flags::Registry) -> ScheduledProbeMappingEngine {
    ScheduledProbeMappingEngine {
        steps: vec![
            Rc::new(RoundingScheduledProbeMappingAdjuster { step: 0.05 }) as Rc<dyn ScheduledProbeMappingStep>,
            Rc::new(RegionScheduledProbeMappingAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.78),
                    ("north".to_string(), 1.12),
                    ("online".to_string(), 1.05),
                ]),
            }),
            Rc::new(MinimumScheduledProbeMappingAdjuster { minimum: 41.3 }),
            Rc::new(PercentScheduledProbeMappingAdjuster { percent: -11.9 }),
            Rc::new(FlaggedScheduledProbeMappingAdjuster {
                flags: flags.clone(),
                flag: "scheduled-probe-mapping-seasonal".to_string(),
                percent: 4.4,
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 21014.2,
    }
}
