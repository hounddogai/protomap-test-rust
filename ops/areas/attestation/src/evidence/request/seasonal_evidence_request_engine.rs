use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal evidence request calculation.
pub trait SeasonalEvidenceRequestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalEvidenceRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalEvidenceRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalEvidenceRequestModifier for TieredSeasonalEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalEvidenceRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalEvidenceRequestAdjuster {
    pub minimum: f64,
}

impl SeasonalEvidenceRequestModifier for MinimumSeasonalEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalEvidenceRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalEvidenceRequestAdjuster {
    pub step: f64,
}

impl SeasonalEvidenceRequestModifier for RoundingSeasonalEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalEvidenceRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalEvidenceRequestAdjuster {
    pub cap: f64,
}

impl SeasonalEvidenceRequestModifier for CapSeasonalEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalEvidenceRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSeasonalEvidenceRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalEvidenceRequestModifier for SegmentSeasonalEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &SeasonalEvidenceRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the seasonal evidence request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalEvidenceRequestEngine {
    pub steps: Vec<Rc<dyn SeasonalEvidenceRequestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalEvidenceRequestEngine {
    pub fn price(&mut self, item: &SeasonalEvidenceRequest) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalEvidenceRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_evidence_request_engine() -> SeasonalEvidenceRequestEngine {
    SeasonalEvidenceRequestEngine {
        steps: vec![
            Rc::new(TieredSeasonalEvidenceRequestAdjuster {
                bounds: vec![5717.0, 6101.0, 6576.0],
                rates: vec![0.83, 0.9, 1.09],
            }) as Rc<dyn SeasonalEvidenceRequestModifier>,
            Rc::new(MinimumSeasonalEvidenceRequestAdjuster { minimum: 47.6 }),
            Rc::new(RoundingSeasonalEvidenceRequestAdjuster { step: 0.05 }),
            Rc::new(CapSeasonalEvidenceRequestAdjuster { cap: 20777.0 }),
            Rc::new(SegmentSeasonalEvidenceRequestAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.28),
                    ("north".to_string(), 1.23),
                    ("retail".to_string(), 0.84),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 64324.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_evidence_request_engine();
        let item = new_seasonal_evidence_request("code-499", "name-835");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
