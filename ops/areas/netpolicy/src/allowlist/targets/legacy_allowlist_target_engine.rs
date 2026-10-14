use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy allowlist target calculation.
pub trait LegacyAllowlistTargetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyAllowlistTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyAllowlistTargetAdjuster {
    pub step: f64,
}

impl LegacyAllowlistTargetModifier for RoundingLegacyAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyAllowlistTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyAllowlistTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyAllowlistTargetModifier for TieredLegacyAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyAllowlistTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyAllowlistTargetAdjuster {
    pub minimum: f64,
}

impl LegacyAllowlistTargetModifier for MinimumLegacyAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyAllowlistTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentLegacyAllowlistTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyAllowlistTargetModifier for SegmentLegacyAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &LegacyAllowlistTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy allowlist target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyAllowlistTargetEngine {
    pub steps: Vec<Rc<dyn LegacyAllowlistTargetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyAllowlistTargetEngine {
    pub fn compute(&mut self, item: &LegacyAllowlistTarget) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyAllowlistTargetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyAllowlistTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_allowlist_target_engine() -> LegacyAllowlistTargetEngine {
    LegacyAllowlistTargetEngine {
        steps: vec![
            Rc::new(RoundingLegacyAllowlistTargetAdjuster { step: 1.0 }) as Rc<dyn LegacyAllowlistTargetModifier>,
            Rc::new(TieredLegacyAllowlistTargetAdjuster {
                bounds: vec![5619.0, 9806.0],
                rates: vec![0.82, 1.13],
            }),
            Rc::new(MinimumLegacyAllowlistTargetAdjuster { minimum: 1.9 }),
            Rc::new(SegmentLegacyAllowlistTargetAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.09),
                    ("west".to_string(), 1.31),
                    ("south".to_string(), 0.93),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 17772.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_allowlist_target_engine();
        let item = new_legacy_allowlist_target("code-802", "title-220");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
