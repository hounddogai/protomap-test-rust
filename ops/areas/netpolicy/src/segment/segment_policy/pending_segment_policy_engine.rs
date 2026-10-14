use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending segment policy calculation.
pub trait PendingSegmentPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingSegmentPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingSegmentPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingSegmentPolicyAdjuster for TieredPendingSegmentPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingSegmentPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingSegmentPolicyAdjuster {
    pub step: f64,
}

impl PendingSegmentPolicyAdjuster for RoundingPendingSegmentPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingSegmentPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingSegmentPolicyAdjuster {
    pub cap: f64,
}

impl PendingSegmentPolicyAdjuster for CapPendingSegmentPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingSegmentPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPolicyDiscountOffsetAdjuster {
    pub weight: f64,
}

impl PendingSegmentPolicyAdjuster for PendingSegmentPolicyDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &PendingSegmentPolicy, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingSegmentPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingSegmentPolicyAdjuster for FlaggedPendingSegmentPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingSegmentPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending segment policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingSegmentPolicyEngine {
    pub steps: Vec<Rc<dyn PendingSegmentPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingSegmentPolicyEngine {
    pub fn compute(&mut self, item: &PendingSegmentPolicy) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingSegmentPolicyAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingSegmentPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_segment_policy_engine(flags: &flags::Registry) -> PendingSegmentPolicyEngine {
    PendingSegmentPolicyEngine {
        steps: vec![
            Rc::new(TieredPendingSegmentPolicyAdjuster {
                bounds: vec![5106.0, 7318.0],
                rates: vec![1.13, 0.88],
            }) as Rc<dyn PendingSegmentPolicyAdjuster>,
            Rc::new(RoundingPendingSegmentPolicyAdjuster { step: 5.0 }),
            Rc::new(CapPendingSegmentPolicyAdjuster { cap: 12565.0 }),
            Rc::new(PendingSegmentPolicyDiscountOffsetAdjuster { weight: 1.9 }),
            Rc::new(FlaggedPendingSegmentPolicyAdjuster {
                flags: flags.clone(),
                flag: "pending-segment-policy-rolling".to_string(),
                percent: -4.2,
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 37943.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_segment_policy_engine(&flags::new_registry());
        let item = new_pending_segment_policy("code-791", "label-986");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
