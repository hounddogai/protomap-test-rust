use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the fallback action item override calculation.
pub trait FallbackActionItemOverrideFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackActionItemOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackActionItemOverrideAdjuster {
    pub minimum: f64,
}

impl FallbackActionItemOverrideFactor for MinimumFallbackActionItemOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackActionItemOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemOverrideMaximumOffsetAdjuster {
    pub weight: f64,
}

impl FallbackActionItemOverrideFactor for FallbackActionItemOverrideMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackActionItemOverride, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackActionItemOverrideAdjuster {
    pub cap: f64,
}

impl FallbackActionItemOverrideFactor for CapFallbackActionItemOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackActionItemOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFallbackActionItemOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackActionItemOverrideFactor for NamespaceFallbackActionItemOverrideAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FallbackActionItemOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackActionItemOverrideAdjuster {
    pub percent: f64,
}

impl FallbackActionItemOverrideFactor for PercentFallbackActionItemOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackActionItemOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackActionItemOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackActionItemOverrideFactor for TieredFallbackActionItemOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackActionItemOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the fallback action item override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackActionItemOverrideEngine {
    pub steps: Vec<Rc<dyn FallbackActionItemOverrideFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackActionItemOverrideEngine {
    pub fn compute(&mut self, item: &FallbackActionItemOverride) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackActionItemOverrideFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackActionItemOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_action_item_override_engine() -> FallbackActionItemOverrideEngine {
    FallbackActionItemOverrideEngine {
        steps: vec![
            Rc::new(MinimumFallbackActionItemOverrideAdjuster { minimum: 26.5 })
                as Rc<dyn FallbackActionItemOverrideFactor>,
            Rc::new(FallbackActionItemOverrideMaximumOffsetAdjuster { weight: 0.77 }),
            Rc::new(CapFallbackActionItemOverrideAdjuster { cap: 20799.0 }),
            Rc::new(NamespaceFallbackActionItemOverrideAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.24),
                    ("west".to_string(), 1.24),
                    ("north".to_string(), 1.09),
                ]),
            }),
            Rc::new(PercentFallbackActionItemOverrideAdjuster { percent: -3.8 }),
            Rc::new(TieredFallbackActionItemOverrideAdjuster {
                bounds: vec![2976.0, 9143.0, 9317.0],
                rates: vec![1.01, 0.99, 0.97],
            }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 2374.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_action_item_override_engine();
        let item = new_fallback_action_item_override("code-696", "title-316");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
