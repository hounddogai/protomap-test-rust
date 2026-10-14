use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback rollout bundle calculation.
pub trait FallbackRolloutBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRolloutBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackRolloutBundleAdjuster {
    pub cap: f64,
}

impl FallbackRolloutBundleFactor for CapFallbackRolloutBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackRolloutBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackRolloutBundleAdjuster {
    pub step: f64,
}

impl FallbackRolloutBundleFactor for RoundingFallbackRolloutBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackRolloutBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackRolloutBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackRolloutBundleFactor for FlaggedFallbackRolloutBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackRolloutBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback rollout bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRolloutBundleEngine {
    pub steps: Vec<Rc<dyn FallbackRolloutBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRolloutBundleEngine {
    pub fn run(&mut self, item: &FallbackRolloutBundle) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackRolloutBundleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRolloutBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_rollout_bundle_engine(flags: &flags::Registry) -> FallbackRolloutBundleEngine {
    FallbackRolloutBundleEngine {
        steps: vec![
            Rc::new(CapFallbackRolloutBundleAdjuster { cap: 10436.0 }) as Rc<dyn FallbackRolloutBundleFactor>,
            Rc::new(RoundingFallbackRolloutBundleAdjuster { step: 0.5 }),
            Rc::new(FlaggedFallbackRolloutBundleAdjuster {
                flags: flags.clone(),
                flag: "fallback-rollout-bundle-global".to_string(),
                percent: 4.9,
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 65466.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_rollout_bundle_engine(&flags::new_registry());
        let item = new_fallback_rollout_bundle("code-87", "name-150");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
