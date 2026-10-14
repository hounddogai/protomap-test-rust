use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the credential request calculation.
pub trait CredentialRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CredentialRequestStep for TieredCredentialRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CredentialRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRequestAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl CredentialRequestStep for CredentialRequestAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &CredentialRequest, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelCredentialRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialRequestStep for LabelCredentialRequestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &CredentialRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCredentialRequestAdjuster {
    pub minimum: f64,
}

impl CredentialRequestStep for MinimumCredentialRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CredentialRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCredentialRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CredentialRequestStep for FlaggedCredentialRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CredentialRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the credential request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialRequestEngine {
    pub steps: Vec<Rc<dyn CredentialRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialRequestEngine {
    pub fn compute(&mut self, item: &CredentialRequest) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CredentialRequestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CredentialRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_credential_request_engine(flags: &flags::Registry) -> CredentialRequestEngine {
    CredentialRequestEngine {
        steps: vec![
            Rc::new(TieredCredentialRequestAdjuster {
                bounds: vec![2378.0, 6910.0, 9559.0],
                rates: vec![1.08, 0.8, 0.84],
            }) as Rc<dyn CredentialRequestStep>,
            Rc::new(CredentialRequestAttemptsOffsetAdjuster { weight: 1.89 }),
            Rc::new(LabelCredentialRequestAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.84),
                    ("west".to_string(), 0.97),
                    ("east".to_string(), 1.29),
                ]),
            }),
            Rc::new(MinimumCredentialRequestAdjuster { minimum: 36.4 }),
            Rc::new(FlaggedCredentialRequestAdjuster {
                flags: flags.clone(),
                flag: "credential-request-manual".to_string(),
                percent: -2.6,
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 43925.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_request_engine(&flags::new_registry());
        let item = new_credential_request("code-156", "label-900");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
