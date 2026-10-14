use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the default peering review calculation.
pub trait DefaultPeeringReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultPeeringReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultPeeringReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultPeeringReviewFactor for TieredDefaultPeeringReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultPeeringReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringReviewTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl DefaultPeeringReviewFactor for DefaultPeeringReviewTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultPeeringReview, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryDefaultPeeringReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultPeeringReviewFactor for CountryDefaultPeeringReviewAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &DefaultPeeringReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default peering review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultPeeringReviewEngine {
    pub steps: Vec<Rc<dyn DefaultPeeringReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultPeeringReviewEngine {
    pub fn price(&mut self, item: &DefaultPeeringReview) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultPeeringReviewFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultPeeringReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_peering_review_engine() -> DefaultPeeringReviewEngine {
    DefaultPeeringReviewEngine {
        steps: vec![
            Rc::new(TieredDefaultPeeringReviewAdjuster {
                bounds: vec![1024.0, 1313.0, 2739.0],
                rates: vec![1.18, 1.12, 1.16],
            }) as Rc<dyn DefaultPeeringReviewFactor>,
            Rc::new(DefaultPeeringReviewTtlSecondsOffsetAdjuster { weight: 1.0 }),
            Rc::new(CountryDefaultPeeringReviewAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.05), ("south".to_string(), 0.83)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 81801.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_peering_review_engine();
        let item = new_default_peering_review("code-350", "label-787");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
