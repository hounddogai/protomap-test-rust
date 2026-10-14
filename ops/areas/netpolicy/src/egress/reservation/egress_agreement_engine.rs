use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the egress agreement calculation.
pub trait EgressAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressAgreementStep for FlaggedEgressAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EgressAgreementStep for TieredEgressAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EgressAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAgreementRateOffsetAdjuster {
    pub weight: f64,
}

impl EgressAgreementStep for EgressAgreementRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &EgressAgreement, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEgressAgreementAdjuster {
    pub cap: f64,
}

impl EgressAgreementStep for CapEgressAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EgressAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressAgreementAdjuster {
    pub percent: f64,
}

impl EgressAgreementStep for PercentEgressAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the egress agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressAgreementEngine {
    pub steps: Vec<Rc<dyn EgressAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressAgreementEngine {
    pub fn evaluate(&mut self, item: &EgressAgreement) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EgressAgreementStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EgressAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_egress_agreement_engine(flags: &flags::Registry) -> EgressAgreementEngine {
    EgressAgreementEngine {
        steps: vec![
            Rc::new(FlaggedEgressAgreementAdjuster {
                flags: flags.clone(),
                flag: "egress-agreement-rolling".to_string(),
                percent: -2.1,
            }) as Rc<dyn EgressAgreementStep>,
            Rc::new(TieredEgressAgreementAdjuster {
                bounds: vec![1665.0, 2544.0, 5193.0, 6798.0],
                rates: vec![0.9, 0.98, 0.87, 0.88],
            }),
            Rc::new(EgressAgreementRateOffsetAdjuster { weight: 0.67 }),
            Rc::new(CapEgressAgreementAdjuster { cap: 16605.0 }),
            Rc::new(PercentEgressAgreementAdjuster { percent: 17.0 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 38441.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_agreement_engine(&flags::new_registry());
        let item = new_egress_agreement("code-384", "label-412");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
