use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual egress bundle calculation.
pub trait ManualEgressBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualEgressBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualEgressBundleAdjuster {
    pub percent: f64,
}

impl ManualEgressBundleModifier for PercentManualEgressBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualEgressBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualEgressBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualEgressBundleModifier for TieredManualEgressBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualEgressBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualEgressBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualEgressBundleModifier for FlaggedManualEgressBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualEgressBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEgressBundleBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl ManualEgressBundleModifier for ManualEgressBundleBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &ManualEgressBundle, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualEgressBundleAdjuster {
    pub step: f64,
}

impl ManualEgressBundleModifier for RoundingManualEgressBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualEgressBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual egress bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualEgressBundleEngine {
    pub steps: Vec<Rc<dyn ManualEgressBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualEgressBundleEngine {
    pub fn run(&mut self, item: &ManualEgressBundle) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualEgressBundleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualEgressBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_egress_bundle_engine(flags: &flags::Registry) -> ManualEgressBundleEngine {
    ManualEgressBundleEngine {
        steps: vec![
            Rc::new(PercentManualEgressBundleAdjuster { percent: -2.3 }) as Rc<dyn ManualEgressBundleModifier>,
            Rc::new(TieredManualEgressBundleAdjuster {
                bounds: vec![6486.0, 8414.0],
                rates: vec![1.09, 0.82],
            }),
            Rc::new(FlaggedManualEgressBundleAdjuster {
                flags: flags.clone(),
                flag: "manual-egress-bundle-strict".to_string(),
                percent: 4.3,
            }),
            Rc::new(ManualEgressBundleBatchSizeOffsetAdjuster { weight: 1.78 }),
            Rc::new(RoundingManualEgressBundleAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 56021.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_egress_bundle_engine(&flags::new_registry());
        let item = new_manual_egress_bundle("code-491", "title-987");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
