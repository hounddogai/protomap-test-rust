use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending flow log profile calculation.
pub trait PendingFlowLogProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingFlowLogProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingFlowLogProfileAdjuster {
    pub step: f64,
}

impl PendingFlowLogProfileModifier for RoundingPendingFlowLogProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingFlowLogProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingFlowLogProfileAdjuster {
    pub percent: f64,
}

impl PendingFlowLogProfileModifier for PercentPendingFlowLogProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingFlowLogProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingFlowLogProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingFlowLogProfileModifier for TieredPendingFlowLogProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingFlowLogProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingFlowLogProfileAdjuster {
    pub minimum: f64,
}

impl PendingFlowLogProfileModifier for MinimumPendingFlowLogProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingFlowLogProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingFlowLogProfileAdjuster {
    pub cap: f64,
}

impl PendingFlowLogProfileModifier for CapPendingFlowLogProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingFlowLogProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogProfileScoreOffsetAdjuster {
    pub weight: f64,
}

impl PendingFlowLogProfileModifier for PendingFlowLogProfileScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &PendingFlowLogProfile, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the pending flow log profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingFlowLogProfileEngine {
    pub steps: Vec<Rc<dyn PendingFlowLogProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingFlowLogProfileEngine {
    pub fn run(&mut self, item: &PendingFlowLogProfile) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingFlowLogProfileModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingFlowLogProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_flow_log_profile_engine() -> PendingFlowLogProfileEngine {
    PendingFlowLogProfileEngine {
        steps: vec![
            Rc::new(RoundingPendingFlowLogProfileAdjuster { step: 0.25 }) as Rc<dyn PendingFlowLogProfileModifier>,
            Rc::new(PercentPendingFlowLogProfileAdjuster { percent: 25.0 }),
            Rc::new(TieredPendingFlowLogProfileAdjuster {
                bounds: vec![2857.0, 5152.0, 7702.0],
                rates: vec![0.82, 1.13, 1.06],
            }),
            Rc::new(MinimumPendingFlowLogProfileAdjuster { minimum: 41.6 }),
            Rc::new(CapPendingFlowLogProfileAdjuster { cap: 18933.0 }),
            Rc::new(PendingFlowLogProfileScoreOffsetAdjuster { weight: 0.88 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 11663.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_flow_log_profile_engine();
        let item = new_pending_flow_log_profile("code-289", "title-623");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
