use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback binding tier calculation.
pub trait FallbackBindingTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackBindingTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingTierUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl FallbackBindingTierFactor for FallbackBindingTierUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackBindingTier, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackBindingTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackBindingTierFactor for TieredFallbackBindingTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackBindingTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackBindingTierAdjuster {
    pub step: f64,
}

impl FallbackBindingTierFactor for RoundingFallbackBindingTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackBindingTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackBindingTierAdjuster {
    pub percent: f64,
}

impl FallbackBindingTierFactor for PercentFallbackBindingTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackBindingTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackBindingTierAdjuster {
    pub cap: f64,
}

impl FallbackBindingTierFactor for CapFallbackBindingTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackBindingTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFallbackBindingTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackBindingTierFactor for LabelFallbackBindingTierAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FallbackBindingTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback binding tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackBindingTierEngine {
    pub steps: Vec<Rc<dyn FallbackBindingTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackBindingTierEngine {
    pub fn evaluate(&mut self, item: &FallbackBindingTier) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[FallbackBindingTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_binding_tier_engine() -> FallbackBindingTierEngine {
    FallbackBindingTierEngine {
        steps: vec![
            Rc::new(FallbackBindingTierUpdatedAtOffsetAdjuster { weight: 0.85 }) as Rc<dyn FallbackBindingTierFactor>,
            Rc::new(TieredFallbackBindingTierAdjuster {
                bounds: vec![1327.0, 4537.0, 9854.0],
                rates: vec![1.0, 1.13, 1.09],
            }),
            Rc::new(RoundingFallbackBindingTierAdjuster { step: 1.0 }),
            Rc::new(PercentFallbackBindingTierAdjuster { percent: -11.7 }),
            Rc::new(CapFallbackBindingTierAdjuster { cap: 34726.0 }),
            Rc::new(LabelFallbackBindingTierAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.35), ("retail".to_string(), 1.18)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 32484.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_binding_tier_engine();
        let item = new_fallback_binding_tier("code-666", "label-125");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
