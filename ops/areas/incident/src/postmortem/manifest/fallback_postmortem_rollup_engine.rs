use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback postmortem rollup calculation.
pub trait FallbackPostmortemRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPostmortemRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackPostmortemRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackPostmortemRollupModifier for TieredFallbackPostmortemRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackPostmortemRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackPostmortemRollupAdjuster {
    pub minimum: f64,
}

impl FallbackPostmortemRollupModifier for MinimumFallbackPostmortemRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackPostmortemRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemRollupGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl FallbackPostmortemRollupModifier for FallbackPostmortemRollupGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackPostmortemRollup, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the fallback postmortem rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPostmortemRollupEngine {
    pub steps: Vec<Rc<dyn FallbackPostmortemRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPostmortemRollupEngine {
    pub fn compute(&mut self, item: &FallbackPostmortemRollup) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[FallbackPostmortemRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_postmortem_rollup_engine() -> FallbackPostmortemRollupEngine {
    FallbackPostmortemRollupEngine {
        steps: vec![
            Rc::new(TieredFallbackPostmortemRollupAdjuster {
                bounds: vec![1010.0, 9163.0, 9764.0],
                rates: vec![1.09, 1.0, 1.15],
            }) as Rc<dyn FallbackPostmortemRollupModifier>,
            Rc::new(MinimumFallbackPostmortemRollupAdjuster { minimum: 44.5 }),
            Rc::new(FallbackPostmortemRollupGraceDaysOffsetAdjuster { weight: 1.39 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 45501.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_postmortem_rollup_engine();
        let item = new_fallback_postmortem_rollup("code-176", "name-738");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
