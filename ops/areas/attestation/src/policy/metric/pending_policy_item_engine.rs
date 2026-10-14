use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending policy item calculation.
pub trait PendingPolicyItemModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingPolicyItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingPolicyItemAdjuster {
    pub cap: f64,
}

impl PendingPolicyItemModifier for CapPendingPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingPolicyItem, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingPolicyItemAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingPolicyItemModifier for TieredPendingPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingPolicyItem, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingPolicyItemAdjuster {
    pub percent: f64,
}

impl PendingPolicyItemModifier for PercentPendingPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingPolicyItem, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingPolicyItemAdjuster {
    pub step: f64,
}

impl PendingPolicyItemModifier for RoundingPendingPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingPolicyItem, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingPolicyItemAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingPolicyItemModifier for FlaggedPendingPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingPolicyItem, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingPolicyItemAdjuster {
    pub minimum: f64,
}

impl PendingPolicyItemModifier for MinimumPendingPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingPolicyItem, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending policy item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingPolicyItemEngine {
    pub steps: Vec<Rc<dyn PendingPolicyItemModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingPolicyItemEngine {
    pub fn price(&mut self, item: &PendingPolicyItem) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[PendingPolicyItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_policy_item_engine(flags: &flags::Registry) -> PendingPolicyItemEngine {
    PendingPolicyItemEngine {
        steps: vec![
            Rc::new(CapPendingPolicyItemAdjuster { cap: 49688.0 }) as Rc<dyn PendingPolicyItemModifier>,
            Rc::new(TieredPendingPolicyItemAdjuster {
                bounds: vec![8308.0, 8309.0, 8647.0],
                rates: vec![0.88, 0.86, 1.05],
            }),
            Rc::new(PercentPendingPolicyItemAdjuster { percent: 12.1 }),
            Rc::new(RoundingPendingPolicyItemAdjuster { step: 0.05 }),
            Rc::new(FlaggedPendingPolicyItemAdjuster {
                flags: flags.clone(),
                flag: "pending-policy-item-weighted".to_string(),
                percent: -1.3,
            }),
            Rc::new(MinimumPendingPolicyItemAdjuster { minimum: 46.0 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 29042.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_policy_item_engine(&flags::new_registry());
        let item = new_pending_policy_item("code-225", "name-998");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
