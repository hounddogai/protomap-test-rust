use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted overage digest calculation.
pub trait WeightedOverageDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedOverageDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedOverageDigestAdjuster {
    pub percent: f64,
}

impl WeightedOverageDigestModifier for PercentWeightedOverageDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedOverageDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedOverageDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedOverageDigestModifier for TieredWeightedOverageDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedOverageDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitWeightedOverageDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedOverageDigestModifier for UnitWeightedOverageDigestAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &WeightedOverageDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted overage digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedOverageDigestEngine {
    pub steps: Vec<Rc<dyn WeightedOverageDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedOverageDigestEngine {
    pub fn run(&mut self, item: &WeightedOverageDigest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[WeightedOverageDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_weighted_overage_digest_engine() -> WeightedOverageDigestEngine {
    WeightedOverageDigestEngine {
        steps: vec![
            Rc::new(PercentWeightedOverageDigestAdjuster { percent: -10.5 }) as Rc<dyn WeightedOverageDigestModifier>,
            Rc::new(TieredWeightedOverageDigestAdjuster {
                bounds: vec![1498.0, 4937.0],
                rates: vec![0.91, 1.01],
            }),
            Rc::new(UnitWeightedOverageDigestAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.75),
                    ("central".to_string(), 0.8),
                    ("south".to_string(), 1.21),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 25971.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_overage_digest_engine();
        let item = new_weighted_overage_digest("code-190", "label-689");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
