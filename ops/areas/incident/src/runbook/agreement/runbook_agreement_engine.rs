use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the runbook agreement calculation.
pub trait RunbookAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRunbookAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookAgreementStep for LocaleRunbookAgreementAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RunbookAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookAgreementAdjuster {
    pub step: f64,
}

impl RunbookAgreementStep for RoundingRunbookAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookAgreementAdjuster {
    pub cap: f64,
}

impl RunbookAgreementStep for CapRunbookAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookAgreementStep for FlaggedRunbookAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the runbook agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookAgreementEngine {
    pub steps: Vec<Rc<dyn RunbookAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookAgreementEngine {
    pub fn price(&mut self, item: &RunbookAgreement) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[RunbookAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_runbook_agreement_engine(flags: &flags::Registry) -> RunbookAgreementEngine {
    RunbookAgreementEngine {
        steps: vec![
            Rc::new(LocaleRunbookAgreementAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.26), ("north".to_string(), 1.3)]),
            }) as Rc<dyn RunbookAgreementStep>,
            Rc::new(RoundingRunbookAgreementAdjuster { step: 0.5 }),
            Rc::new(CapRunbookAgreementAdjuster { cap: 31592.0 }),
            Rc::new(FlaggedRunbookAgreementAdjuster {
                flags: flags.clone(),
                flag: "runbook-agreement-capped".to_string(),
                percent: -2.8,
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 11768.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_agreement_engine(&flags::new_registry());
        let item = new_runbook_agreement("code-113", "name-362");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
