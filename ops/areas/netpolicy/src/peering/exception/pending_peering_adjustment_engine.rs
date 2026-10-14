use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the pending peering adjustment calculation.
pub trait PendingPeeringAdjustmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingPeeringAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPeeringAdjustmentAmountOffsetAdjuster {
    pub weight: f64,
}

impl PendingPeeringAdjustmentStep for PendingPeeringAdjustmentAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &PendingPeeringAdjustment, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingPeeringAdjustmentAdjuster {
    pub cap: f64,
}

impl PendingPeeringAdjustmentStep for CapPendingPeeringAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingPeeringAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingPeeringAdjustmentAdjuster {
    pub percent: f64,
}

impl PendingPeeringAdjustmentStep for PercentPendingPeeringAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingPeeringAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingPeeringAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingPeeringAdjustmentStep for TieredPendingPeeringAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingPeeringAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterPendingPeeringAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingPeeringAdjustmentStep for ClusterPendingPeeringAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &PendingPeeringAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingPeeringAdjustmentAdjuster {
    pub minimum: f64,
}

impl PendingPeeringAdjustmentStep for MinimumPendingPeeringAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingPeeringAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending peering adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingPeeringAdjustmentEngine {
    pub steps: Vec<Rc<dyn PendingPeeringAdjustmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingPeeringAdjustmentEngine {
    pub fn run(&mut self, item: &PendingPeeringAdjustment) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingPeeringAdjustmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingPeeringAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_peering_adjustment_engine() -> PendingPeeringAdjustmentEngine {
    PendingPeeringAdjustmentEngine {
        steps: vec![
            Rc::new(PendingPeeringAdjustmentAmountOffsetAdjuster { weight: 0.81 })
                as Rc<dyn PendingPeeringAdjustmentStep>,
            Rc::new(CapPendingPeeringAdjustmentAdjuster { cap: 9811.0 }),
            Rc::new(PercentPendingPeeringAdjustmentAdjuster { percent: 23.6 }),
            Rc::new(TieredPendingPeeringAdjustmentAdjuster {
                bounds: vec![1438.0, 2838.0, 4520.0, 7202.0],
                rates: vec![1.18, 0.83, 1.2, 1.13],
            }),
            Rc::new(ClusterPendingPeeringAdjustmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.82)]),
            }),
            Rc::new(MinimumPendingPeeringAdjustmentAdjuster { minimum: 13.2 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 21719.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_peering_adjustment_engine();
        let item = new_pending_peering_adjustment("code-501", "name-576");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
