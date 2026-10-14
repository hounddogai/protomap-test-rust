use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict overage agreement calculation.
pub trait StrictOverageAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictOverageAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictOverageAgreementAdjuster {
    pub percent: f64,
}

impl StrictOverageAgreementModifier for PercentStrictOverageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictOverageAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictOverageAgreementAdjuster {
    pub cap: f64,
}

impl StrictOverageAgreementModifier for CapStrictOverageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictOverageAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictOverageAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictOverageAgreementModifier for FlaggedStrictOverageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictOverageAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictOverageAgreementAdjuster {
    pub step: f64,
}

impl StrictOverageAgreementModifier for RoundingStrictOverageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictOverageAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameStrictOverageAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictOverageAgreementModifier for NameStrictOverageAgreementAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &StrictOverageAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageAgreementExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl StrictOverageAgreementModifier for StrictOverageAgreementExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &StrictOverageAgreement, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

/// Runs the strict overage agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictOverageAgreementEngine {
    pub steps: Vec<Rc<dyn StrictOverageAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictOverageAgreementEngine {
    pub fn compute(&mut self, item: &StrictOverageAgreement) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictOverageAgreementModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictOverageAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_overage_agreement_engine(flags: &flags::Registry) -> StrictOverageAgreementEngine {
    StrictOverageAgreementEngine {
        steps: vec![
            Rc::new(PercentStrictOverageAgreementAdjuster { percent: 17.0 }) as Rc<dyn StrictOverageAgreementModifier>,
            Rc::new(CapStrictOverageAgreementAdjuster { cap: 14624.0 }),
            Rc::new(FlaggedStrictOverageAgreementAdjuster {
                flags: flags.clone(),
                flag: "strict-overage-agreement-seasonal".to_string(),
                percent: -8.6,
            }),
            Rc::new(RoundingStrictOverageAgreementAdjuster { step: 5.0 }),
            Rc::new(NameStrictOverageAgreementAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.79), ("online".to_string(), 1.28)]),
            }),
            Rc::new(StrictOverageAgreementExpiresAtOffsetAdjuster { weight: 1.06 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 46625.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_overage_agreement_engine(&flags::new_registry());
        let item = new_strict_overage_agreement("code-310", "name-595");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
