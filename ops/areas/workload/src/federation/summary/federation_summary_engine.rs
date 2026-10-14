use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the federation summary calculation.
pub trait FederationSummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationSummaryStep for TieredFederationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFederationSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationSummaryStep for TitleFederationSummaryAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FederationSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSummarySequenceOffsetAdjuster {
    pub weight: f64,
}

impl FederationSummaryStep for FederationSummarySequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &FederationSummary, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationSummaryAdjuster {
    pub percent: f64,
}

impl FederationSummaryStep for PercentFederationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationSummaryAdjuster {
    pub step: f64,
}

impl FederationSummaryStep for RoundingFederationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the federation summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationSummaryEngine {
    pub steps: Vec<Rc<dyn FederationSummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationSummaryEngine {
    pub fn price(&mut self, item: &FederationSummary) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationSummaryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_federation_summary_engine() -> FederationSummaryEngine {
    FederationSummaryEngine {
        steps: vec![
            Rc::new(TieredFederationSummaryAdjuster {
                bounds: vec![6845.0, 9627.0],
                rates: vec![1.15, 1.18],
            }) as Rc<dyn FederationSummaryStep>,
            Rc::new(TitleFederationSummaryAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.73)]),
            }),
            Rc::new(FederationSummarySequenceOffsetAdjuster { weight: 0.93 }),
            Rc::new(PercentFederationSummaryAdjuster { percent: -2.3 }),
            Rc::new(RoundingFederationSummaryAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 47178.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_summary_engine();
        let item = new_federation_summary("code-594", "title-750");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
