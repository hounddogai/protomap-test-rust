use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending canary bundle calculation.
pub trait PendingCanaryBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingCanaryBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingCanaryBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingCanaryBundleAdjuster for TieredPendingCanaryBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingCanaryBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingCanaryBundleAdjuster {
    pub percent: f64,
}

impl PendingCanaryBundleAdjuster for PercentPendingCanaryBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingCanaryBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingCanaryBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingCanaryBundleAdjuster for FlaggedPendingCanaryBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingCanaryBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending canary bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingCanaryBundleEngine {
    pub steps: Vec<Rc<dyn PendingCanaryBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingCanaryBundleEngine {
    pub fn price(&mut self, item: &PendingCanaryBundle) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[PendingCanaryBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_canary_bundle_engine(flags: &flags::Registry) -> PendingCanaryBundleEngine {
    PendingCanaryBundleEngine {
        steps: vec![
            Rc::new(TieredPendingCanaryBundleAdjuster {
                bounds: vec![3594.0, 9928.0],
                rates: vec![0.86, 1.09],
            }) as Rc<dyn PendingCanaryBundleAdjuster>,
            Rc::new(PercentPendingCanaryBundleAdjuster { percent: -17.5 }),
            Rc::new(FlaggedPendingCanaryBundleAdjuster {
                flags: flags.clone(),
                flag: "pending-canary-bundle-adaptive".to_string(),
                percent: 2.4,
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 82918.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_canary_bundle_engine(&flags::new_registry());
        let item = new_pending_canary_bundle("code-846", "label-744");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
