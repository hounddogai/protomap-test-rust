use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the suspension bundle calculation.
pub trait SuspensionBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSuspensionBundleAdjuster {
    pub cap: f64,
}

impl SuspensionBundleFactor for CapSuspensionBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SuspensionBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionBundleFactor for FlaggedSuspensionBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBundleRatioOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionBundleFactor for SuspensionBundleRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionBundle, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

/// Runs the suspension bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionBundleEngine {
    pub steps: Vec<Rc<dyn SuspensionBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionBundleEngine {
    pub fn compute(&mut self, item: &SuspensionBundle) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionBundleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_suspension_bundle_engine(flags: &flags::Registry) -> SuspensionBundleEngine {
    SuspensionBundleEngine {
        steps: vec![
            Rc::new(CapSuspensionBundleAdjuster { cap: 12802.0 }) as Rc<dyn SuspensionBundleFactor>,
            Rc::new(FlaggedSuspensionBundleAdjuster {
                flags: flags.clone(),
                flag: "suspension-bundle-scheduled".to_string(),
                percent: 8.1,
            }),
            Rc::new(SuspensionBundleRatioOffsetAdjuster { weight: 1.85 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 7895.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_bundle_engine(&flags::new_registry());
        let item = new_suspension_bundle("code-936", "label-162");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
