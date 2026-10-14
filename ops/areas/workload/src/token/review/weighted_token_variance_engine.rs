use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the weighted token variance calculation.
pub trait WeightedTokenVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedTokenVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryWeightedTokenVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedTokenVarianceStep for CategoryWeightedTokenVarianceAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &WeightedTokenVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedTokenVarianceAdjuster {
    pub percent: f64,
}

impl WeightedTokenVarianceStep for PercentWeightedTokenVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedTokenVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenVarianceWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl WeightedTokenVarianceStep for WeightedTokenVarianceWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedTokenVariance, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedTokenVarianceAdjuster {
    pub minimum: f64,
}

impl WeightedTokenVarianceStep for MinimumWeightedTokenVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedTokenVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedTokenVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedTokenVarianceStep for TieredWeightedTokenVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedTokenVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted token variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedTokenVarianceEngine {
    pub steps: Vec<Rc<dyn WeightedTokenVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedTokenVarianceEngine {
    pub fn price(&mut self, item: &WeightedTokenVariance) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedTokenVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedTokenVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_token_variance_engine() -> WeightedTokenVarianceEngine {
    WeightedTokenVarianceEngine {
        steps: vec![
            Rc::new(CategoryWeightedTokenVarianceAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.11),
                    ("north".to_string(), 1.12),
                    ("east".to_string(), 1.28),
                ]),
            }) as Rc<dyn WeightedTokenVarianceStep>,
            Rc::new(PercentWeightedTokenVarianceAdjuster { percent: -4.2 }),
            Rc::new(WeightedTokenVarianceWindowDaysOffsetAdjuster { weight: 0.29 }),
            Rc::new(MinimumWeightedTokenVarianceAdjuster { minimum: 17.8 }),
            Rc::new(TieredWeightedTokenVarianceAdjuster {
                bounds: vec![794.0, 5589.0, 7375.0, 8977.0],
                rates: vec![1.18, 1.07, 1.18, 0.91],
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 42985.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_token_variance_engine();
        let item = new_weighted_token_variance("code-129", "name-871");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
