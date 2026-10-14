use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual retention bundle calculation.
pub trait ManualRetentionBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualRetentionBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionManualRetentionBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualRetentionBundleAdjuster for RegionManualRetentionBundleAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ManualRetentionBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualRetentionBundleAdjuster {
    pub step: f64,
}

impl ManualRetentionBundleAdjuster for RoundingManualRetentionBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualRetentionBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualRetentionBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualRetentionBundleAdjuster for FlaggedManualRetentionBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualRetentionBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualRetentionBundleAdjuster {
    pub percent: f64,
}

impl ManualRetentionBundleAdjuster for PercentManualRetentionBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualRetentionBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualRetentionBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualRetentionBundleAdjuster for TieredManualRetentionBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualRetentionBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual retention bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualRetentionBundleEngine {
    pub steps: Vec<Rc<dyn ManualRetentionBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualRetentionBundleEngine {
    pub fn run(&mut self, item: &ManualRetentionBundle) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ManualRetentionBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_retention_bundle_engine(flags: &flags::Registry) -> ManualRetentionBundleEngine {
    ManualRetentionBundleEngine {
        steps: vec![
            Rc::new(RegionManualRetentionBundleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.06), ("central".to_string(), 0.91)]),
            }) as Rc<dyn ManualRetentionBundleAdjuster>,
            Rc::new(RoundingManualRetentionBundleAdjuster { step: 1.0 }),
            Rc::new(FlaggedManualRetentionBundleAdjuster {
                flags: flags.clone(),
                flag: "manual-retention-bundle-default".to_string(),
                percent: -0.8,
            }),
            Rc::new(PercentManualRetentionBundleAdjuster { percent: 9.6 }),
            Rc::new(TieredManualRetentionBundleAdjuster {
                bounds: vec![3202.0, 4318.0, 8348.0, 9785.0],
                rates: vec![0.85, 0.96, 1.17, 1.06],
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 68676.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_retention_bundle_engine(&flags::new_registry());
        let item = new_manual_retention_bundle("code-733", "label-639");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
