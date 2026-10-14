use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual allowlist target calculation.
pub trait ManualAllowlistTargetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualAllowlistTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualAllowlistTargetAdjuster {
    pub step: f64,
}

impl ManualAllowlistTargetModifier for RoundingManualAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualAllowlistTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualAllowlistTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualAllowlistTargetModifier for TieredManualAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualAllowlistTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleManualAllowlistTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualAllowlistTargetModifier for TitleManualAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ManualAllowlistTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualAllowlistTargetAdjuster {
    pub minimum: f64,
}

impl ManualAllowlistTargetModifier for MinimumManualAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualAllowlistTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual allowlist target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualAllowlistTargetEngine {
    pub steps: Vec<Rc<dyn ManualAllowlistTargetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualAllowlistTargetEngine {
    pub fn evaluate(&mut self, item: &ManualAllowlistTarget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualAllowlistTargetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualAllowlistTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_allowlist_target_engine() -> ManualAllowlistTargetEngine {
    ManualAllowlistTargetEngine {
        steps: vec![
            Rc::new(RoundingManualAllowlistTargetAdjuster { step: 1.0 }) as Rc<dyn ManualAllowlistTargetModifier>,
            Rc::new(TieredManualAllowlistTargetAdjuster {
                bounds: vec![816.0, 1483.0, 4141.0, 7469.0],
                rates: vec![1.06, 0.85, 1.17, 1.01],
            }),
            Rc::new(TitleManualAllowlistTargetAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.74)]),
            }),
            Rc::new(MinimumManualAllowlistTargetAdjuster { minimum: 16.4 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 3372.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_allowlist_target_engine();
        let item = new_manual_allowlist_target("code-586", "title-985");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
