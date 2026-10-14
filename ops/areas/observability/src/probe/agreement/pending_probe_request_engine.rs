use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending probe request calculation.
pub trait PendingProbeRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingProbeRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingProbeRequestAdjuster {
    pub minimum: f64,
}

impl PendingProbeRequestStep for MinimumPendingProbeRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingProbeRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingProbeRequestAdjuster {
    pub percent: f64,
}

impl PendingProbeRequestStep for PercentPendingProbeRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingProbeRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingProbeRequestAdjuster {
    pub step: f64,
}

impl PendingProbeRequestStep for RoundingPendingProbeRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingProbeRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingProbeRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingProbeRequestStep for TieredPendingProbeRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingProbeRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingProbeRequestAdjuster {
    pub cap: f64,
}

impl PendingProbeRequestStep for CapPendingProbeRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingProbeRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeRequestCapacityOffsetAdjuster {
    pub weight: f64,
}

impl PendingProbeRequestStep for PendingProbeRequestCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &PendingProbeRequest, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

/// Runs the pending probe request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingProbeRequestEngine {
    pub steps: Vec<Rc<dyn PendingProbeRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingProbeRequestEngine {
    pub fn run(&mut self, item: &PendingProbeRequest) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingProbeRequestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingProbeRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_probe_request_engine() -> PendingProbeRequestEngine {
    PendingProbeRequestEngine {
        steps: vec![
            Rc::new(MinimumPendingProbeRequestAdjuster { minimum: 48.6 }) as Rc<dyn PendingProbeRequestStep>,
            Rc::new(PercentPendingProbeRequestAdjuster { percent: 17.6 }),
            Rc::new(RoundingPendingProbeRequestAdjuster { step: 5.0 }),
            Rc::new(TieredPendingProbeRequestAdjuster {
                bounds: vec![1798.0, 4140.0, 7087.0],
                rates: vec![1.2, 1.02, 1.16],
            }),
            Rc::new(CapPendingProbeRequestAdjuster { cap: 34746.0 }),
            Rc::new(PendingProbeRequestCapacityOffsetAdjuster { weight: 0.06 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 77012.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_probe_request_engine();
        let item = new_pending_probe_request("code-690", "title-588");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
