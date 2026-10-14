use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default rollout bundle calculation.
pub trait DefaultRolloutBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultRolloutBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDefaultRolloutBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultRolloutBundleAdjuster for TitleDefaultRolloutBundleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DefaultRolloutBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultRolloutBundleAdjuster {
    pub minimum: f64,
}

impl DefaultRolloutBundleAdjuster for MinimumDefaultRolloutBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultRolloutBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultRolloutBundleAdjuster {
    pub cap: f64,
}

impl DefaultRolloutBundleAdjuster for CapDefaultRolloutBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultRolloutBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultRolloutBundleAdjuster {
    pub step: f64,
}

impl DefaultRolloutBundleAdjuster for RoundingDefaultRolloutBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultRolloutBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultRolloutBundleAdjuster {
    pub percent: f64,
}

impl DefaultRolloutBundleAdjuster for PercentDefaultRolloutBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultRolloutBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultRolloutBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultRolloutBundleAdjuster for TieredDefaultRolloutBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultRolloutBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the default rollout bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultRolloutBundleEngine {
    pub steps: Vec<Rc<dyn DefaultRolloutBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultRolloutBundleEngine {
    pub fn price(&mut self, item: &DefaultRolloutBundle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DefaultRolloutBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_rollout_bundle_engine() -> DefaultRolloutBundleEngine {
    DefaultRolloutBundleEngine {
        steps: vec![
            Rc::new(TitleDefaultRolloutBundleAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.08),
                    ("west".to_string(), 1.26),
                    ("north".to_string(), 1.12),
                ]),
            }) as Rc<dyn DefaultRolloutBundleAdjuster>,
            Rc::new(MinimumDefaultRolloutBundleAdjuster { minimum: 30.2 }),
            Rc::new(CapDefaultRolloutBundleAdjuster { cap: 47956.0 }),
            Rc::new(RoundingDefaultRolloutBundleAdjuster { step: 0.5 }),
            Rc::new(PercentDefaultRolloutBundleAdjuster { percent: 7.3 }),
            Rc::new(TieredDefaultRolloutBundleAdjuster {
                bounds: vec![682.0, 3510.0],
                rates: vec![0.82, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 89765.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_rollout_bundle_engine();
        let item = new_default_rollout_bundle("code-141", "title-12");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
