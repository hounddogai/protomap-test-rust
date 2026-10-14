use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive runbook bundle calculation.
pub trait AdaptiveRunbookBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRunbookBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveRunbookBundleAdjuster {
    pub step: f64,
}

impl AdaptiveRunbookBundleFactor for RoundingAdaptiveRunbookBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveRunbookBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveRunbookBundleAdjuster {
    pub percent: f64,
}

impl AdaptiveRunbookBundleFactor for PercentAdaptiveRunbookBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveRunbookBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookBundleVersionOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveRunbookBundleFactor for AdaptiveRunbookBundleVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveRunbookBundle, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleAdaptiveRunbookBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveRunbookBundleFactor for LocaleAdaptiveRunbookBundleAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &AdaptiveRunbookBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveRunbookBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveRunbookBundleFactor for TieredAdaptiveRunbookBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveRunbookBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive runbook bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRunbookBundleEngine {
    pub steps: Vec<Rc<dyn AdaptiveRunbookBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRunbookBundleEngine {
    pub fn run(&mut self, item: &AdaptiveRunbookBundle) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveRunbookBundleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRunbookBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_runbook_bundle_engine() -> AdaptiveRunbookBundleEngine {
    AdaptiveRunbookBundleEngine {
        steps: vec![
            Rc::new(RoundingAdaptiveRunbookBundleAdjuster { step: 0.1 }) as Rc<dyn AdaptiveRunbookBundleFactor>,
            Rc::new(PercentAdaptiveRunbookBundleAdjuster { percent: -0.9 }),
            Rc::new(AdaptiveRunbookBundleVersionOffsetAdjuster { weight: 0.92 }),
            Rc::new(LocaleAdaptiveRunbookBundleAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.37),
                    ("online".to_string(), 1.18),
                    ("retail".to_string(), 1.14),
                ]),
            }),
            Rc::new(TieredAdaptiveRunbookBundleAdjuster {
                bounds: vec![743.0, 4954.0, 7489.0, 8922.0],
                rates: vec![1.15, 1.07, 1.04, 0.91],
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 1318.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_runbook_bundle_engine();
        let item = new_adaptive_runbook_bundle("code-668", "name-679");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
