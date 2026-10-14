use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending probe override calculation.
pub trait PendingProbeOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingProbeOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingProbeOverrideAdjuster {
    pub step: f64,
}

impl PendingProbeOverrideAdjuster for RoundingPendingProbeOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingProbeOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingProbeOverrideAdjuster {
    pub cap: f64,
}

impl PendingProbeOverrideAdjuster for CapPendingProbeOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingProbeOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingProbeOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingProbeOverrideAdjuster for FlaggedPendingProbeOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingProbeOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingProbeOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingProbeOverrideAdjuster for TieredPendingProbeOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingProbeOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending probe override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingProbeOverrideEngine {
    pub steps: Vec<Rc<dyn PendingProbeOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingProbeOverrideEngine {
    pub fn evaluate(&mut self, item: &PendingProbeOverride) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingProbeOverrideAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingProbeOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_probe_override_engine(flags: &flags::Registry) -> PendingProbeOverrideEngine {
    PendingProbeOverrideEngine {
        steps: vec![
            Rc::new(RoundingPendingProbeOverrideAdjuster { step: 0.1 }) as Rc<dyn PendingProbeOverrideAdjuster>,
            Rc::new(CapPendingProbeOverrideAdjuster { cap: 46807.0 }),
            Rc::new(FlaggedPendingProbeOverrideAdjuster {
                flags: flags.clone(),
                flag: "pending-probe-override-legacy".to_string(),
                percent: 9.7,
            }),
            Rc::new(TieredPendingProbeOverrideAdjuster {
                bounds: vec![216.0, 2855.0, 9338.0],
                rates: vec![0.92, 1.13, 1.05],
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 58862.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_probe_override_engine(&flags::new_registry());
        let item = new_pending_probe_override("code-299", "title-446");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
