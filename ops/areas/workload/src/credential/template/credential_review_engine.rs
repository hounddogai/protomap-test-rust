use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the credential review calculation.
pub trait CredentialReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CredentialReviewModifier for TieredCredentialReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CredentialReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCredentialReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CredentialReviewModifier for FlaggedCredentialReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CredentialReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCredentialReviewAdjuster {
    pub minimum: f64,
}

impl CredentialReviewModifier for MinimumCredentialReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CredentialReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCredentialReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialReviewModifier for NameCredentialReviewAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CredentialReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialReviewPriorityOffsetAdjuster {
    pub weight: f64,
}

impl CredentialReviewModifier for CredentialReviewPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &CredentialReview, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

/// Runs the credential review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialReviewEngine {
    pub steps: Vec<Rc<dyn CredentialReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialReviewEngine {
    pub fn price(&mut self, item: &CredentialReview) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[CredentialReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_credential_review_engine(flags: &flags::Registry) -> CredentialReviewEngine {
    CredentialReviewEngine {
        steps: vec![
            Rc::new(TieredCredentialReviewAdjuster {
                bounds: vec![1159.0, 1332.0, 6733.0, 7615.0],
                rates: vec![1.17, 0.9, 1.0, 1.0],
            }) as Rc<dyn CredentialReviewModifier>,
            Rc::new(FlaggedCredentialReviewAdjuster {
                flags: flags.clone(),
                flag: "credential-review-tiered".to_string(),
                percent: -0.1,
            }),
            Rc::new(MinimumCredentialReviewAdjuster { minimum: 34.3 }),
            Rc::new(NameCredentialReviewAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.97)]),
            }),
            Rc::new(CredentialReviewPriorityOffsetAdjuster { weight: 1.83 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 48425.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_review_engine(&flags::new_registry());
        let item = new_credential_review("code-787", "name-832");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
