use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the probe summary calculation.
pub trait ProbeSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeSummaryAdjuster {
    pub percent: f64,
}

impl ProbeSummaryModifier for PercentProbeSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSummaryTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl ProbeSummaryModifier for ProbeSummaryTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &ProbeSummary, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbeSummaryModifier for TieredProbeSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbeSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumProbeSummaryAdjuster {
    pub minimum: f64,
}

impl ProbeSummaryModifier for MinimumProbeSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ProbeSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryProbeSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ProbeSummaryModifier for CategoryProbeSummaryAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &ProbeSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the probe summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeSummaryEngine {
    pub steps: Vec<Rc<dyn ProbeSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeSummaryEngine {
    pub fn compute(&mut self, item: &ProbeSummary) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[ProbeSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_probe_summary_engine() -> ProbeSummaryEngine {
    ProbeSummaryEngine {
        steps: vec![
            Rc::new(PercentProbeSummaryAdjuster { percent: -9.7 }) as Rc<dyn ProbeSummaryModifier>,
            Rc::new(ProbeSummaryTtlSecondsOffsetAdjuster { weight: 0.25 }),
            Rc::new(TieredProbeSummaryAdjuster {
                bounds: vec![5126.0, 5901.0, 7988.0, 8910.0],
                rates: vec![1.19, 0.86, 0.99, 1.02],
            }),
            Rc::new(MinimumProbeSummaryAdjuster { minimum: 12.4 }),
            Rc::new(CategoryProbeSummaryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.39)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 84028.7,
    }
}
