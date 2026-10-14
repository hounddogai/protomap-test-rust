use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default rollout variance calculation.
pub trait DefaultRolloutVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultRolloutVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultRolloutVarianceAdjuster {
    pub minimum: f64,
}

impl DefaultRolloutVarianceStep for MinimumDefaultRolloutVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultRolloutVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultRolloutVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultRolloutVarianceStep for TieredDefaultRolloutVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultRolloutVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutVarianceRateOffsetAdjuster {
    pub weight: f64,
}

impl DefaultRolloutVarianceStep for DefaultRolloutVarianceRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultRolloutVariance, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryDefaultRolloutVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultRolloutVarianceStep for CategoryDefaultRolloutVarianceAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &DefaultRolloutVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default rollout variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultRolloutVarianceEngine {
    pub steps: Vec<Rc<dyn DefaultRolloutVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultRolloutVarianceEngine {
    pub fn compute(&mut self, item: &DefaultRolloutVariance) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[DefaultRolloutVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_rollout_variance_engine() -> DefaultRolloutVarianceEngine {
    DefaultRolloutVarianceEngine {
        steps: vec![
            Rc::new(MinimumDefaultRolloutVarianceAdjuster { minimum: 33.8 }) as Rc<dyn DefaultRolloutVarianceStep>,
            Rc::new(TieredDefaultRolloutVarianceAdjuster {
                bounds: vec![2626.0, 4057.0, 5220.0, 5374.0],
                rates: vec![0.93, 1.0, 0.82, 0.81],
            }),
            Rc::new(DefaultRolloutVarianceRateOffsetAdjuster { weight: 0.56 }),
            Rc::new(CategoryDefaultRolloutVarianceAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.39),
                    ("retail".to_string(), 1.09),
                    ("west".to_string(), 1.26),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 17134.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_rollout_variance_engine();
        let item = new_default_rollout_variance("code-445", "title-501");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
