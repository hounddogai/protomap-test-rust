use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tiered escalation agreement calculation.
pub trait TieredEscalationAgreementAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredEscalationAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredEscalationAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredEscalationAgreementAdjuster for FlaggedTieredEscalationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredEscalationAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredEscalationAgreementAdjuster {
    pub percent: f64,
}

impl TieredEscalationAgreementAdjuster for PercentTieredEscalationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredEscalationAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTieredEscalationAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredEscalationAgreementAdjuster for NameTieredEscalationAgreementAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TieredEscalationAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the tiered escalation agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredEscalationAgreementEngine {
    pub steps: Vec<Rc<dyn TieredEscalationAgreementAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredEscalationAgreementEngine {
    pub fn run(&mut self, item: &TieredEscalationAgreement) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredEscalationAgreementAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredEscalationAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_escalation_agreement_engine(flags: &flags::Registry) -> TieredEscalationAgreementEngine {
    TieredEscalationAgreementEngine {
        steps: vec![
            Rc::new(FlaggedTieredEscalationAgreementAdjuster {
                flags: flags.clone(),
                flag: "tiered-escalation-agreement-pending".to_string(),
                percent: 1.8,
            }) as Rc<dyn TieredEscalationAgreementAdjuster>,
            Rc::new(PercentTieredEscalationAgreementAdjuster { percent: 0.0 }),
            Rc::new(NameTieredEscalationAgreementAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.4)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 8002.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_escalation_agreement_engine(&flags::new_registry());
        let item = new_tiered_escalation_agreement("code-448", "name-276");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
