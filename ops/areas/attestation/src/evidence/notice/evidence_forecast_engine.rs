use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the evidence forecast calculation.
pub trait EvidenceForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceEvidenceForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceForecastFactor for NamespaceEvidenceForecastAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &EvidenceForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceForecastScoreOffsetAdjuster {
    pub weight: f64,
}

impl EvidenceForecastFactor for EvidenceForecastScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &EvidenceForecast, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceForecastFactor for TieredEvidenceForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceForecastAdjuster {
    pub minimum: f64,
}

impl EvidenceForecastFactor for MinimumEvidenceForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceForecastAdjuster {
    pub percent: f64,
}

impl EvidenceForecastFactor for PercentEvidenceForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the evidence forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceForecastEngine {
    pub steps: Vec<Rc<dyn EvidenceForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceForecastEngine {
    pub fn evaluate(&mut self, item: &EvidenceForecast) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceForecastFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_evidence_forecast_engine() -> EvidenceForecastEngine {
    EvidenceForecastEngine {
        steps: vec![
            Rc::new(NamespaceEvidenceForecastAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.18),
                    ("retail".to_string(), 1.35),
                    ("central".to_string(), 0.84),
                ]),
            }) as Rc<dyn EvidenceForecastFactor>,
            Rc::new(EvidenceForecastScoreOffsetAdjuster { weight: 1.81 }),
            Rc::new(TieredEvidenceForecastAdjuster {
                bounds: vec![1136.0, 6618.0],
                rates: vec![1.02, 1.19],
            }),
            Rc::new(MinimumEvidenceForecastAdjuster { minimum: 41.2 }),
            Rc::new(PercentEvidenceForecastAdjuster { percent: 22.6 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 56513.2,
    }
}
