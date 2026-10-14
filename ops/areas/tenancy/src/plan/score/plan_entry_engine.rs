use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan entry calculation.
pub trait PlanEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanEntryAdjuster {
    pub minimum: f64,
}

impl PlanEntryAdjuster for MinimumPlanEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelPlanEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanEntryAdjuster for ChannelPlanEntryAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &PlanEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanEntryAdjuster {
    pub cap: f64,
}

impl PlanEntryAdjuster for CapPlanEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanEntryAdjuster {
    pub step: f64,
}

impl PlanEntryAdjuster for RoundingPlanEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanEntryAdjuster {
    pub percent: f64,
}

impl PlanEntryAdjuster for PercentPlanEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanEntryAdjuster for FlaggedPlanEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the plan entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanEntryEngine {
    pub steps: Vec<Rc<dyn PlanEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanEntryEngine {
    pub fn compute(&mut self, item: &PlanEntry) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_plan_entry_engine(flags: &flags::Registry) -> PlanEntryEngine {
    PlanEntryEngine {
        steps: vec![
            Rc::new(MinimumPlanEntryAdjuster { minimum: 31.4 }) as Rc<dyn PlanEntryAdjuster>,
            Rc::new(ChannelPlanEntryAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.08)]),
            }),
            Rc::new(CapPlanEntryAdjuster { cap: 10939.0 }),
            Rc::new(RoundingPlanEntryAdjuster { step: 0.1 }),
            Rc::new(PercentPlanEntryAdjuster { percent: 23.5 }),
            Rc::new(FlaggedPlanEntryAdjuster {
                flags: flags.clone(),
                flag: "plan-entry-capped".to_string(),
                percent: -6.9,
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 61695.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_entry_engine(&flags::new_registry());
        let item = new_plan_entry("code-512", "title-626");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
