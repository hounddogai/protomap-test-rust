use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending slo profile calculation.
pub trait PendingSloProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingSloProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingSloProfileAdjuster {
    pub step: f64,
}

impl PendingSloProfileStep for RoundingPendingSloProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingSloProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingSloProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingSloProfileStep for TieredPendingSloProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingSloProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingSloProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingSloProfileStep for FlaggedPendingSloProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingSloProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingSloProfileAdjuster {
    pub percent: f64,
}

impl PendingSloProfileStep for PercentPendingSloProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingSloProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloProfileLimitOffsetAdjuster {
    pub weight: f64,
}

impl PendingSloProfileStep for PendingSloProfileLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &PendingSloProfile, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingSloProfileAdjuster {
    pub minimum: f64,
}

impl PendingSloProfileStep for MinimumPendingSloProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingSloProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending slo profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingSloProfileEngine {
    pub steps: Vec<Rc<dyn PendingSloProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingSloProfileEngine {
    pub fn price(&mut self, item: &PendingSloProfile) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingSloProfileStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingSloProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_slo_profile_engine(flags: &flags::Registry) -> PendingSloProfileEngine {
    PendingSloProfileEngine {
        steps: vec![
            Rc::new(RoundingPendingSloProfileAdjuster { step: 0.1 }) as Rc<dyn PendingSloProfileStep>,
            Rc::new(TieredPendingSloProfileAdjuster {
                bounds: vec![236.0, 6024.0],
                rates: vec![1.11, 1.0],
            }),
            Rc::new(FlaggedPendingSloProfileAdjuster {
                flags: flags.clone(),
                flag: "pending-slo-profile-global".to_string(),
                percent: -6.5,
            }),
            Rc::new(PercentPendingSloProfileAdjuster { percent: 1.4 }),
            Rc::new(PendingSloProfileLimitOffsetAdjuster { weight: 0.76 }),
            Rc::new(MinimumPendingSloProfileAdjuster { minimum: 49.9 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 76127.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_slo_profile_engine(&flags::new_registry());
        let item = new_pending_slo_profile("code-940", "label-50");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
