use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal escalation agreement calculation.
pub trait SeasonalEscalationAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalEscalationAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalEscalationAgreementAdjuster {
    pub minimum: f64,
}

impl SeasonalEscalationAgreementStep for MinimumSeasonalEscalationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalEscalationAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalEscalationAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalEscalationAgreementStep for FlaggedSeasonalEscalationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalEscalationAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalEscalationAgreementAdjuster {
    pub step: f64,
}

impl SeasonalEscalationAgreementStep for RoundingSeasonalEscalationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalEscalationAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalEscalationAgreementAdjuster {
    pub cap: f64,
}

impl SeasonalEscalationAgreementStep for CapSeasonalEscalationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalEscalationAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEscalationAgreementBudgetOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalEscalationAgreementStep for SeasonalEscalationAgreementBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalEscalationAgreement, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalEscalationAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalEscalationAgreementStep for TieredSeasonalEscalationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalEscalationAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the seasonal escalation agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalEscalationAgreementEngine {
    pub steps: Vec<Rc<dyn SeasonalEscalationAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalEscalationAgreementEngine {
    pub fn price(&mut self, item: &SeasonalEscalationAgreement) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalEscalationAgreementStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalEscalationAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_escalation_agreement_engine(flags: &flags::Registry) -> SeasonalEscalationAgreementEngine {
    SeasonalEscalationAgreementEngine {
        steps: vec![
            Rc::new(MinimumSeasonalEscalationAgreementAdjuster { minimum: 43.3 })
                as Rc<dyn SeasonalEscalationAgreementStep>,
            Rc::new(FlaggedSeasonalEscalationAgreementAdjuster {
                flags: flags.clone(),
                flag: "seasonal-escalation-agreement-primary".to_string(),
                percent: 1.1,
            }),
            Rc::new(RoundingSeasonalEscalationAgreementAdjuster { step: 1.0 }),
            Rc::new(CapSeasonalEscalationAgreementAdjuster { cap: 2098.0 }),
            Rc::new(SeasonalEscalationAgreementBudgetOffsetAdjuster { weight: 0.3 }),
            Rc::new(TieredSeasonalEscalationAgreementAdjuster {
                bounds: vec![1815.0, 2193.0, 4857.0, 9588.0],
                rates: vec![0.92, 1.1, 0.95, 0.91],
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 29862.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_escalation_agreement_engine(&flags::new_registry());
        let item = new_seasonal_escalation_agreement("code-711", "name-873");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
