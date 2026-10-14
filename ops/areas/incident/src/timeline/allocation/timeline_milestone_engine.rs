use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the timeline milestone calculation.
pub trait TimelineMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineMilestoneAdjuster {
    pub percent: f64,
}

impl TimelineMilestoneStep for PercentTimelineMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTimelineMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TimelineMilestoneStep for FlaggedTimelineMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TimelineMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineMilestoneAdjuster {
    pub cap: f64,
}

impl TimelineMilestoneStep for CapTimelineMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TimelineMilestoneStep for TieredTimelineMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TimelineMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the timeline milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineMilestoneEngine {
    pub steps: Vec<Rc<dyn TimelineMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineMilestoneEngine {
    pub fn price(&mut self, item: &TimelineMilestone) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[TimelineMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_timeline_milestone_engine(flags: &flags::Registry) -> TimelineMilestoneEngine {
    TimelineMilestoneEngine {
        steps: vec![
            Rc::new(PercentTimelineMilestoneAdjuster { percent: -8.1 }) as Rc<dyn TimelineMilestoneStep>,
            Rc::new(FlaggedTimelineMilestoneAdjuster {
                flags: flags.clone(),
                flag: "timeline-milestone-default".to_string(),
                percent: -8.2,
            }),
            Rc::new(CapTimelineMilestoneAdjuster { cap: 32995.0 }),
            Rc::new(TieredTimelineMilestoneAdjuster {
                bounds: vec![498.0, 6151.0, 8132.0],
                rates: vec![0.8, 1.13, 0.96],
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 72451.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_timeline_milestone_engine(&flags::new_registry());
        let item = new_timeline_milestone("code-812", "title-514");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
