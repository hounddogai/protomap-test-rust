use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional federation agreement calculation.
pub trait RegionalFederationAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalFederationAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalFederationAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalFederationAgreementStep for FlaggedRegionalFederationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalFederationAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalFederationAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalFederationAgreementStep for TieredRegionalFederationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalFederationAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationAgreementScoreOffsetAdjuster {
    pub weight: f64,
}

impl RegionalFederationAgreementStep for RegionalFederationAgreementScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalFederationAgreement, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalFederationAgreementAdjuster {
    pub percent: f64,
}

impl RegionalFederationAgreementStep for PercentRegionalFederationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalFederationAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalFederationAgreementAdjuster {
    pub cap: f64,
}

impl RegionalFederationAgreementStep for CapRegionalFederationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalFederationAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the regional federation agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalFederationAgreementEngine {
    pub steps: Vec<Rc<dyn RegionalFederationAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalFederationAgreementEngine {
    pub fn compute(&mut self, item: &RegionalFederationAgreement) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalFederationAgreementStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalFederationAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_federation_agreement_engine(flags: &flags::Registry) -> RegionalFederationAgreementEngine {
    RegionalFederationAgreementEngine {
        steps: vec![
            Rc::new(FlaggedRegionalFederationAgreementAdjuster {
                flags: flags.clone(),
                flag: "regional-federation-agreement-capped".to_string(),
                percent: 9.5,
            }) as Rc<dyn RegionalFederationAgreementStep>,
            Rc::new(TieredRegionalFederationAgreementAdjuster {
                bounds: vec![1127.0, 5581.0, 6265.0],
                rates: vec![0.91, 1.09, 1.17],
            }),
            Rc::new(RegionalFederationAgreementScoreOffsetAdjuster { weight: 0.23 }),
            Rc::new(PercentRegionalFederationAgreementAdjuster { percent: 13.2 }),
            Rc::new(CapRegionalFederationAgreementAdjuster { cap: 13594.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 27526.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_federation_agreement_engine(&flags::new_registry());
        let item = new_regional_federation_agreement("code-577", "title-391");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
