use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the pending probe adjustment calculation.
pub trait PendingProbeAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingProbeAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAdjustmentCapacityOffsetAdjuster {
    pub weight: f64,
}

impl PendingProbeAdjustmentFactor for PendingProbeAdjustmentCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &PendingProbeAdjustment, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingProbeAdjustmentAdjuster {
    pub cap: f64,
}

impl PendingProbeAdjustmentFactor for CapPendingProbeAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingProbeAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingProbeAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingProbeAdjustmentFactor for TieredPendingProbeAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingProbeAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingProbeAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingProbeAdjustmentFactor for FlaggedPendingProbeAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingProbeAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending probe adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingProbeAdjustmentEngine {
    pub steps: Vec<Rc<dyn PendingProbeAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingProbeAdjustmentEngine {
    pub fn compute(&mut self, item: &PendingProbeAdjustment) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingProbeAdjustmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingProbeAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_probe_adjustment_engine(flags: &flags::Registry) -> PendingProbeAdjustmentEngine {
    PendingProbeAdjustmentEngine {
        steps: vec![
            Rc::new(PendingProbeAdjustmentCapacityOffsetAdjuster { weight: 0.28 })
                as Rc<dyn PendingProbeAdjustmentFactor>,
            Rc::new(CapPendingProbeAdjustmentAdjuster { cap: 21108.0 }),
            Rc::new(TieredPendingProbeAdjustmentAdjuster {
                bounds: vec![1335.0, 4638.0, 8535.0],
                rates: vec![0.9, 1.03, 1.06],
            }),
            Rc::new(FlaggedPendingProbeAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "pending-probe-adjustment-primary".to_string(),
                percent: -8.6,
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 2659.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_probe_adjustment_engine(&flags::new_registry());
        let item = new_pending_probe_adjustment("code-69", "label-241");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
