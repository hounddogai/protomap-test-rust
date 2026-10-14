use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy rollout window calculation.
pub trait LegacyRolloutWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyRolloutWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyRolloutWindowAdjuster {
    pub cap: f64,
}

impl LegacyRolloutWindowModifier for CapLegacyRolloutWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyRolloutWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutWindowStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl LegacyRolloutWindowModifier for LegacyRolloutWindowStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyRolloutWindow, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterLegacyRolloutWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyRolloutWindowModifier for ClusterLegacyRolloutWindowAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &LegacyRolloutWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyRolloutWindowAdjuster {
    pub minimum: f64,
}

impl LegacyRolloutWindowModifier for MinimumLegacyRolloutWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyRolloutWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyRolloutWindowAdjuster {
    pub step: f64,
}

impl LegacyRolloutWindowModifier for RoundingLegacyRolloutWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyRolloutWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the legacy rollout window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyRolloutWindowEngine {
    pub steps: Vec<Rc<dyn LegacyRolloutWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyRolloutWindowEngine {
    pub fn evaluate(&mut self, item: &LegacyRolloutWindow) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyRolloutWindowModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyRolloutWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_rollout_window_engine() -> LegacyRolloutWindowEngine {
    LegacyRolloutWindowEngine {
        steps: vec![
            Rc::new(CapLegacyRolloutWindowAdjuster { cap: 4116.0 }) as Rc<dyn LegacyRolloutWindowModifier>,
            Rc::new(LegacyRolloutWindowStartsAtOffsetAdjuster { weight: 0.46 }),
            Rc::new(ClusterLegacyRolloutWindowAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.19)]),
            }),
            Rc::new(MinimumLegacyRolloutWindowAdjuster { minimum: 17.4 }),
            Rc::new(RoundingLegacyRolloutWindowAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 67848.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_rollout_window_engine();
        let item = new_legacy_rollout_window("code-828", "name-869");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
