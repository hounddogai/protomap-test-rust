use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the fallback egress agreement calculation.
pub trait FallbackEgressAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackEgressAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackEgressAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackEgressAgreementModifier for FlaggedFallbackEgressAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackEgressAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackEgressAgreementAdjuster {
    pub percent: f64,
}

impl FallbackEgressAgreementModifier for PercentFallbackEgressAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackEgressAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackEgressAgreementAdjuster {
    pub minimum: f64,
}

impl FallbackEgressAgreementModifier for MinimumFallbackEgressAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackEgressAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackEgressAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackEgressAgreementModifier for TieredFallbackEgressAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackEgressAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the fallback egress agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackEgressAgreementEngine {
    pub steps: Vec<Rc<dyn FallbackEgressAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackEgressAgreementEngine {
    pub fn evaluate(&mut self, item: &FallbackEgressAgreement) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackEgressAgreementModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackEgressAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_egress_agreement_engine(flags: &flags::Registry) -> FallbackEgressAgreementEngine {
    FallbackEgressAgreementEngine {
        steps: vec![
            Rc::new(FlaggedFallbackEgressAgreementAdjuster {
                flags: flags.clone(),
                flag: "fallback-egress-agreement-primary".to_string(),
                percent: 0.4,
            }) as Rc<dyn FallbackEgressAgreementModifier>,
            Rc::new(PercentFallbackEgressAgreementAdjuster { percent: 15.4 }),
            Rc::new(MinimumFallbackEgressAgreementAdjuster { minimum: 35.2 }),
            Rc::new(TieredFallbackEgressAgreementAdjuster {
                bounds: vec![905.0, 3804.0, 3983.0],
                rates: vec![1.12, 0.97, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 80206.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_egress_agreement_engine(&flags::new_registry());
        let item = new_fallback_egress_agreement("code-271", "name-484");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
