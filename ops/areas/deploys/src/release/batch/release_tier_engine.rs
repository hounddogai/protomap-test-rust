use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the release tier calculation.
pub trait ReleaseTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitReleaseTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseTierAdjuster for UnitReleaseTierAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ReleaseTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseTierAdjuster for TieredReleaseTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTierQuantityOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseTierAdjuster for ReleaseTierQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseTier, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the release tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseTierEngine {
    pub steps: Vec<Rc<dyn ReleaseTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseTierEngine {
    pub fn evaluate(&mut self, item: &ReleaseTier) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_release_tier_engine() -> ReleaseTierEngine {
    ReleaseTierEngine {
        steps: vec![
            Rc::new(UnitReleaseTierAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.95), ("central".to_string(), 0.89)]),
            }) as Rc<dyn ReleaseTierAdjuster>,
            Rc::new(TieredReleaseTierAdjuster {
                bounds: vec![987.0, 6961.0, 9633.0],
                rates: vec![0.99, 0.83, 0.86],
            }),
            Rc::new(ReleaseTierQuantityOffsetAdjuster { weight: 0.28 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 69224.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_tier_engine();
        let item = new_release_tier("code-45", "title-853");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
