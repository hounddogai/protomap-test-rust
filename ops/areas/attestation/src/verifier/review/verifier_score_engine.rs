use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the verifier score calculation.
pub trait VerifierScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentVerifierScoreAdjuster {
    pub percent: f64,
}

impl VerifierScoreStep for PercentVerifierScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &VerifierScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl VerifierScoreStep for TieredVerifierScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &VerifierScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierScoreStep for FlaggedVerifierScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierScoreAdjuster {
    pub step: f64,
}

impl VerifierScoreStep for RoundingVerifierScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the verifier score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierScoreEngine {
    pub steps: Vec<Rc<dyn VerifierScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierScoreEngine {
    pub fn compute(&mut self, item: &VerifierScore) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn VerifierScoreStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[VerifierScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_verifier_score_engine(flags: &flags::Registry) -> VerifierScoreEngine {
    VerifierScoreEngine {
        steps: vec![
            Rc::new(PercentVerifierScoreAdjuster { percent: 9.7 }) as Rc<dyn VerifierScoreStep>,
            Rc::new(TieredVerifierScoreAdjuster {
                bounds: vec![4300.0, 5094.0, 5907.0, 8756.0],
                rates: vec![0.89, 1.04, 1.04, 0.96],
            }),
            Rc::new(FlaggedVerifierScoreAdjuster {
                flags: flags.clone(),
                flag: "verifier-score-scheduled".to_string(),
                percent: 9.1,
            }),
            Rc::new(RoundingVerifierScoreAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 73568.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_score_engine(&flags::new_registry());
        let item = new_verifier_score("code-813", "title-478");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
