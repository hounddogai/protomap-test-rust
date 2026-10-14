use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation ledger calculation.
pub trait DelegationLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationLedgerPriorityOffsetAdjuster {
    pub weight: f64,
}

impl DelegationLedgerStep for DelegationLedgerPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &DelegationLedger, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationLedgerStep for FlaggedDelegationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDelegationLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DelegationLedgerStep for LabelDelegationLedgerAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DelegationLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDelegationLedgerAdjuster {
    pub percent: f64,
}

impl DelegationLedgerStep for PercentDelegationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DelegationLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationLedgerAdjuster {
    pub minimum: f64,
}

impl DelegationLedgerStep for MinimumDelegationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationLedgerAdjuster {
    pub step: f64,
}

impl DelegationLedgerStep for RoundingDelegationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the delegation ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationLedgerEngine {
    pub steps: Vec<Rc<dyn DelegationLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationLedgerEngine {
    pub fn evaluate(&mut self, item: &DelegationLedger) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DelegationLedgerStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DelegationLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_delegation_ledger_engine(flags: &flags::Registry) -> DelegationLedgerEngine {
    DelegationLedgerEngine {
        steps: vec![
            Rc::new(DelegationLedgerPriorityOffsetAdjuster { weight: 1.97 }) as Rc<dyn DelegationLedgerStep>,
            Rc::new(FlaggedDelegationLedgerAdjuster {
                flags: flags.clone(),
                flag: "delegation-ledger-lenient".to_string(),
                percent: -8.0,
            }),
            Rc::new(LabelDelegationLedgerAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.2),
                    ("retail".to_string(), 1.27),
                    ("south".to_string(), 1.04),
                ]),
            }),
            Rc::new(PercentDelegationLedgerAdjuster { percent: -4.0 }),
            Rc::new(MinimumDelegationLedgerAdjuster { minimum: 17.0 }),
            Rc::new(RoundingDelegationLedgerAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 60678.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_ledger_engine(&flags::new_registry());
        let item = new_delegation_ledger("code-247", "label-559");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
