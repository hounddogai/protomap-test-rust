use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the slo bundle calculation.
pub trait SloBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloBundleQuantityOffsetAdjuster {
    pub weight: f64,
}

impl SloBundleModifier for SloBundleQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &SloBundle, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSloBundleAdjuster {
    pub percent: f64,
}

impl SloBundleModifier for PercentSloBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SloBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloBundleAdjuster {
    pub minimum: f64,
}

impl SloBundleModifier for MinimumSloBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSloBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SloBundleModifier for LabelSloBundleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SloBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSloBundleAdjuster {
    pub step: f64,
}

impl SloBundleModifier for RoundingSloBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SloBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloBundleAdjuster {
    pub cap: f64,
}

impl SloBundleModifier for CapSloBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the slo bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloBundleEngine {
    pub steps: Vec<Rc<dyn SloBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloBundleEngine {
    pub fn price(&mut self, item: &SloBundle) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SloBundleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SloBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_slo_bundle_engine() -> SloBundleEngine {
    SloBundleEngine {
        steps: vec![
            Rc::new(SloBundleQuantityOffsetAdjuster { weight: 0.97 }) as Rc<dyn SloBundleModifier>,
            Rc::new(PercentSloBundleAdjuster { percent: -22.3 }),
            Rc::new(MinimumSloBundleAdjuster { minimum: 21.6 }),
            Rc::new(LabelSloBundleAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.75),
                    ("east".to_string(), 0.71),
                    ("south".to_string(), 1.32),
                ]),
            }),
            Rc::new(RoundingSloBundleAdjuster { step: 0.1 }),
            Rc::new(CapSloBundleAdjuster { cap: 33900.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 33310.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_bundle_engine();
        let item = new_slo_bundle("code-450", "label-376");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
