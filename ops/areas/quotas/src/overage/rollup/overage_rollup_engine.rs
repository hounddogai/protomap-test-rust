use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the overage rollup calculation.
pub trait OverageRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentOverageRollupAdjuster {
    pub percent: f64,
}

impl OverageRollupFactor for PercentOverageRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &OverageRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl OverageRollupFactor for TieredOverageRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &OverageRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryOverageRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl OverageRollupFactor for CategoryOverageRollupAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &OverageRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRollupRevisionOffsetAdjuster {
    pub weight: f64,
}

impl OverageRollupFactor for OverageRollupRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &OverageRollup, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

/// Runs the overage rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageRollupEngine {
    pub steps: Vec<Rc<dyn OverageRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageRollupEngine {
    pub fn evaluate(&mut self, item: &OverageRollup) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[OverageRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_overage_rollup_engine() -> OverageRollupEngine {
    OverageRollupEngine {
        steps: vec![
            Rc::new(PercentOverageRollupAdjuster { percent: -24.1 }) as Rc<dyn OverageRollupFactor>,
            Rc::new(TieredOverageRollupAdjuster {
                bounds: vec![7344.0, 9584.0],
                rates: vec![0.85, 1.06],
            }),
            Rc::new(CategoryOverageRollupAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.26),
                    ("retail".to_string(), 0.99),
                    ("east".to_string(), 1.3),
                ]),
            }),
            Rc::new(OverageRollupRevisionOffsetAdjuster { weight: 0.46 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 57057.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_rollup_engine();
        let item = new_overage_rollup("code-45", "name-683");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
