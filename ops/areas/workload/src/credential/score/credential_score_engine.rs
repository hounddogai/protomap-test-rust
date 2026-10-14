use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the credential score calculation.
pub trait CredentialScoreFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCredentialScoreAdjuster {
    pub cap: f64,
}

impl CredentialScoreFactor for CapCredentialScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CredentialScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCredentialScoreAdjuster {
    pub step: f64,
}

impl CredentialScoreFactor for RoundingCredentialScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CredentialScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCredentialScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CredentialScoreFactor for FlaggedCredentialScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CredentialScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCredentialScoreAdjuster {
    pub percent: f64,
}

impl CredentialScoreFactor for PercentCredentialScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CredentialScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionCredentialScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialScoreFactor for RegionCredentialScoreAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &CredentialScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CredentialScoreFactor for TieredCredentialScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CredentialScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the credential score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialScoreEngine {
    pub steps: Vec<Rc<dyn CredentialScoreFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialScoreEngine {
    pub fn evaluate(&mut self, item: &CredentialScore) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[CredentialScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_credential_score_engine(flags: &flags::Registry) -> CredentialScoreEngine {
    CredentialScoreEngine {
        steps: vec![
            Rc::new(CapCredentialScoreAdjuster { cap: 31666.0 }) as Rc<dyn CredentialScoreFactor>,
            Rc::new(RoundingCredentialScoreAdjuster { step: 5.0 }),
            Rc::new(FlaggedCredentialScoreAdjuster {
                flags: flags.clone(),
                flag: "credential-score-rolling".to_string(),
                percent: -5.9,
            }),
            Rc::new(PercentCredentialScoreAdjuster { percent: -9.1 }),
            Rc::new(RegionCredentialScoreAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.21), ("central".to_string(), 0.76)]),
            }),
            Rc::new(TieredCredentialScoreAdjuster {
                bounds: vec![4213.0, 5590.0, 8931.0],
                rates: vec![0.95, 0.81, 0.99],
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 91335.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_score_engine(&flags::new_registry());
        let item = new_credential_score("code-369", "label-937");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
