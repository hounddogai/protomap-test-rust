use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the federation exception calculation.
pub trait FederationExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFederationExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationExceptionAdjuster for TitleFederationExceptionAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FederationException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationExceptionAdjuster {
    pub step: f64,
}

impl FederationExceptionAdjuster for RoundingFederationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationExceptionAdjuster for TieredFederationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationExceptionAdjuster {
    pub minimum: f64,
}

impl FederationExceptionAdjuster for MinimumFederationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationExceptionAdjuster {
    pub percent: f64,
}

impl FederationExceptionAdjuster for PercentFederationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationExceptionQuantityOffsetAdjuster {
    pub weight: f64,
}

impl FederationExceptionAdjuster for FederationExceptionQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &FederationException, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the federation exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationExceptionEngine {
    pub steps: Vec<Rc<dyn FederationExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationExceptionEngine {
    pub fn evaluate(&mut self, item: &FederationException) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationExceptionAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_federation_exception_engine() -> FederationExceptionEngine {
    FederationExceptionEngine {
        steps: vec![
            Rc::new(TitleFederationExceptionAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.28),
                    ("west".to_string(), 1.21),
                    ("central".to_string(), 0.72),
                ]),
            }) as Rc<dyn FederationExceptionAdjuster>,
            Rc::new(RoundingFederationExceptionAdjuster { step: 0.05 }),
            Rc::new(TieredFederationExceptionAdjuster {
                bounds: vec![6403.0, 7591.0],
                rates: vec![0.84, 1.19],
            }),
            Rc::new(MinimumFederationExceptionAdjuster { minimum: 12.1 }),
            Rc::new(PercentFederationExceptionAdjuster { percent: -8.3 }),
            Rc::new(FederationExceptionQuantityOffsetAdjuster { weight: 0.6 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 23097.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_exception_engine();
        let item = new_federation_exception("code-905", "title-319");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
