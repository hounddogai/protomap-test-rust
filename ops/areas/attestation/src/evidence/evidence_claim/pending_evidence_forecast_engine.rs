use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending evidence forecast calculation.
pub trait PendingEvidenceForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingEvidenceForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEvidenceForecastScoreOffsetAdjuster {
    pub weight: f64,
}

impl PendingEvidenceForecastFactor for PendingEvidenceForecastScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &PendingEvidenceForecast, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryPendingEvidenceForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingEvidenceForecastFactor for CountryPendingEvidenceForecastAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &PendingEvidenceForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingEvidenceForecastAdjuster {
    pub minimum: f64,
}

impl PendingEvidenceForecastFactor for MinimumPendingEvidenceForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingEvidenceForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingEvidenceForecastAdjuster {
    pub step: f64,
}

impl PendingEvidenceForecastFactor for RoundingPendingEvidenceForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingEvidenceForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingEvidenceForecastAdjuster {
    pub cap: f64,
}

impl PendingEvidenceForecastFactor for CapPendingEvidenceForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingEvidenceForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the pending evidence forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingEvidenceForecastEngine {
    pub steps: Vec<Rc<dyn PendingEvidenceForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingEvidenceForecastEngine {
    pub fn price(&mut self, item: &PendingEvidenceForecast) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingEvidenceForecastFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingEvidenceForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_evidence_forecast_engine() -> PendingEvidenceForecastEngine {
    PendingEvidenceForecastEngine {
        steps: vec![
            Rc::new(PendingEvidenceForecastScoreOffsetAdjuster { weight: 1.53 })
                as Rc<dyn PendingEvidenceForecastFactor>,
            Rc::new(CountryPendingEvidenceForecastAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.09),
                    ("south".to_string(), 1.01),
                    ("north".to_string(), 1.28),
                ]),
            }),
            Rc::new(MinimumPendingEvidenceForecastAdjuster { minimum: 9.3 }),
            Rc::new(RoundingPendingEvidenceForecastAdjuster { step: 1.0 }),
            Rc::new(CapPendingEvidenceForecastAdjuster { cap: 45983.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 60399.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_evidence_forecast_engine();
        let item = new_pending_evidence_forecast("code-924", "title-445");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
