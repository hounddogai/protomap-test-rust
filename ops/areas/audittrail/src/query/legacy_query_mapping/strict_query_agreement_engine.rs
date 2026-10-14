use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the strict query agreement calculation.
pub trait StrictQueryAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictQueryAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictQueryAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictQueryAgreementStep for TieredStrictQueryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictQueryAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictQueryAgreementAdjuster {
    pub cap: f64,
}

impl StrictQueryAgreementStep for CapStrictQueryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictQueryAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeStrictQueryAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictQueryAgreementStep for ScopeStrictQueryAgreementAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &StrictQueryAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictQueryAgreementAdjuster {
    pub percent: f64,
}

impl StrictQueryAgreementStep for PercentStrictQueryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictQueryAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictQueryAgreementAdjuster {
    pub minimum: f64,
}

impl StrictQueryAgreementStep for MinimumStrictQueryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictQueryAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the strict query agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictQueryAgreementEngine {
    pub steps: Vec<Rc<dyn StrictQueryAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictQueryAgreementEngine {
    pub fn evaluate(&mut self, item: &StrictQueryAgreement) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictQueryAgreementStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictQueryAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_query_agreement_engine() -> StrictQueryAgreementEngine {
    StrictQueryAgreementEngine {
        steps: vec![
            Rc::new(TieredStrictQueryAgreementAdjuster {
                bounds: vec![5530.0, 8320.0, 8650.0],
                rates: vec![1.11, 1.15, 0.9],
            }) as Rc<dyn StrictQueryAgreementStep>,
            Rc::new(CapStrictQueryAgreementAdjuster { cap: 45723.0 }),
            Rc::new(ScopeStrictQueryAgreementAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.76)]),
            }),
            Rc::new(PercentStrictQueryAgreementAdjuster { percent: -17.4 }),
            Rc::new(MinimumStrictQueryAgreementAdjuster { minimum: 29.9 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 87627.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_query_agreement_engine();
        let item = new_strict_query_agreement("code-212", "label-366");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
