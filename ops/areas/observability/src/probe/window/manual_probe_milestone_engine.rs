use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual probe milestone calculation.
pub trait ManualProbeMilestoneAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualProbeMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualProbeMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualProbeMilestoneAdjuster for TieredManualProbeMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualProbeMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualProbeMilestoneAdjuster {
    pub percent: f64,
}

impl ManualProbeMilestoneAdjuster for PercentManualProbeMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualProbeMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualProbeMilestoneAdjuster {
    pub minimum: f64,
}

impl ManualProbeMilestoneAdjuster for MinimumManualProbeMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualProbeMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualProbeMilestoneAdjuster {
    pub cap: f64,
}

impl ManualProbeMilestoneAdjuster for CapManualProbeMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualProbeMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeMilestoneRateOffsetAdjuster {
    pub weight: f64,
}

impl ManualProbeMilestoneAdjuster for ManualProbeMilestoneRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &ManualProbeMilestone, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualProbeMilestoneAdjuster {
    pub step: f64,
}

impl ManualProbeMilestoneAdjuster for RoundingManualProbeMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualProbeMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual probe milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualProbeMilestoneEngine {
    pub steps: Vec<Rc<dyn ManualProbeMilestoneAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualProbeMilestoneEngine {
    pub fn evaluate(&mut self, item: &ManualProbeMilestone) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualProbeMilestoneAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualProbeMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_probe_milestone_engine() -> ManualProbeMilestoneEngine {
    ManualProbeMilestoneEngine {
        steps: vec![
            Rc::new(TieredManualProbeMilestoneAdjuster {
                bounds: vec![783.0, 8774.0],
                rates: vec![1.12, 0.87],
            }) as Rc<dyn ManualProbeMilestoneAdjuster>,
            Rc::new(PercentManualProbeMilestoneAdjuster { percent: -15.6 }),
            Rc::new(MinimumManualProbeMilestoneAdjuster { minimum: 20.4 }),
            Rc::new(CapManualProbeMilestoneAdjuster { cap: 29289.0 }),
            Rc::new(ManualProbeMilestoneRateOffsetAdjuster { weight: 0.15 }),
            Rc::new(RoundingManualProbeMilestoneAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 42888.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_probe_milestone_engine();
        let item = new_manual_probe_milestone("code-220", "title-422");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
