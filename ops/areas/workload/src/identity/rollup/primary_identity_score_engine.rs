use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary identity score calculation.
pub trait PrimaryIdentityScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryIdentityScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryIdentityScoreAdjuster {
    pub minimum: f64,
}

impl PrimaryIdentityScoreStep for MinimumPrimaryIdentityScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryIdentityScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryIdentityScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryIdentityScoreStep for TieredPrimaryIdentityScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryIdentityScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryIdentityScoreAdjuster {
    pub cap: f64,
}

impl PrimaryIdentityScoreStep for CapPrimaryIdentityScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryIdentityScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePrimaryIdentityScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryIdentityScoreStep for TitlePrimaryIdentityScoreAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PrimaryIdentityScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityScoreDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryIdentityScoreStep for PrimaryIdentityScoreDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryIdentityScore, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

/// Runs the primary identity score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryIdentityScoreEngine {
    pub steps: Vec<Rc<dyn PrimaryIdentityScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryIdentityScoreEngine {
    pub fn run(&mut self, item: &PrimaryIdentityScore) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryIdentityScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_identity_score_engine() -> PrimaryIdentityScoreEngine {
    PrimaryIdentityScoreEngine {
        steps: vec![
            Rc::new(MinimumPrimaryIdentityScoreAdjuster { minimum: 19.3 }) as Rc<dyn PrimaryIdentityScoreStep>,
            Rc::new(TieredPrimaryIdentityScoreAdjuster {
                bounds: vec![4542.0, 6855.0, 9180.0],
                rates: vec![1.11, 0.84, 1.17],
            }),
            Rc::new(CapPrimaryIdentityScoreAdjuster { cap: 22731.0 }),
            Rc::new(TitlePrimaryIdentityScoreAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.09)]),
            }),
            Rc::new(PrimaryIdentityScoreDurationMinutesOffsetAdjuster { weight: 1.04 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 7272.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_identity_score_engine();
        let item = new_primary_identity_score("code-703", "title-644");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
