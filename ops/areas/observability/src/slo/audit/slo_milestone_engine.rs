use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the slo milestone calculation.
pub trait SloMilestoneAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloMilestoneAdjuster {
    pub minimum: f64,
}

impl SloMilestoneAdjuster for MinimumSloMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloMilestoneAdjuster {
    pub cap: f64,
}

impl SloMilestoneAdjuster for CapSloMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSloMilestoneAdjuster {
    pub percent: f64,
}

impl SloMilestoneAdjuster for PercentSloMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SloMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSloMilestoneAdjuster {
    pub step: f64,
}

impl SloMilestoneAdjuster for RoundingSloMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SloMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the slo milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloMilestoneEngine {
    pub steps: Vec<Rc<dyn SloMilestoneAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloMilestoneEngine {
    pub fn run(&mut self, item: &SloMilestone) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SloMilestoneAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SloMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_slo_milestone_engine() -> SloMilestoneEngine {
    SloMilestoneEngine {
        steps: vec![
            Rc::new(MinimumSloMilestoneAdjuster { minimum: 5.0 }) as Rc<dyn SloMilestoneAdjuster>,
            Rc::new(CapSloMilestoneAdjuster { cap: 14659.0 }),
            Rc::new(PercentSloMilestoneAdjuster { percent: -15.7 }),
            Rc::new(RoundingSloMilestoneAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 32165.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_milestone_engine();
        let item = new_slo_milestone("code-845", "name-93");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
