use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional redaction agreement calculation.
pub trait RegionalRedactionAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalRedactionAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalRedactionAgreementAdjuster {
    pub step: f64,
}

impl RegionalRedactionAgreementModifier for RoundingRegionalRedactionAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalRedactionAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionAgreementPriorityOffsetAdjuster {
    pub weight: f64,
}

impl RegionalRedactionAgreementModifier for RegionalRedactionAgreementPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalRedactionAgreement, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalRedactionAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalRedactionAgreementModifier for FlaggedRegionalRedactionAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalRedactionAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalRedactionAgreementAdjuster {
    pub percent: f64,
}

impl RegionalRedactionAgreementModifier for PercentRegionalRedactionAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalRedactionAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalRedactionAgreementAdjuster {
    pub cap: f64,
}

impl RegionalRedactionAgreementModifier for CapRegionalRedactionAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalRedactionAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the regional redaction agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalRedactionAgreementEngine {
    pub steps: Vec<Rc<dyn RegionalRedactionAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalRedactionAgreementEngine {
    pub fn price(&mut self, item: &RegionalRedactionAgreement) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalRedactionAgreementModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalRedactionAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_redaction_agreement_engine(flags: &flags::Registry) -> RegionalRedactionAgreementEngine {
    RegionalRedactionAgreementEngine {
        steps: vec![
            Rc::new(RoundingRegionalRedactionAgreementAdjuster { step: 0.25 })
                as Rc<dyn RegionalRedactionAgreementModifier>,
            Rc::new(RegionalRedactionAgreementPriorityOffsetAdjuster { weight: 0.72 }),
            Rc::new(FlaggedRegionalRedactionAgreementAdjuster {
                flags: flags.clone(),
                flag: "regional-redaction-agreement-tiered".to_string(),
                percent: -6.6,
            }),
            Rc::new(PercentRegionalRedactionAgreementAdjuster { percent: -14.9 }),
            Rc::new(CapRegionalRedactionAgreementAdjuster { cap: 2971.0 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 55950.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_redaction_agreement_engine(&flags::new_registry());
        let item = new_regional_redaction_agreement("code-916", "name-602");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
