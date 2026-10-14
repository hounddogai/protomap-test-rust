use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the grant ledger calculation.
pub trait GrantLedgerModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGrantLedgerAdjuster {
    pub step: f64,
}

impl GrantLedgerModifier for RoundingGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GrantLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantLedgerAdjuster {
    pub minimum: f64,
}

impl GrantLedgerModifier for MinimumGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelGrantLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantLedgerModifier for ChannelGrantLedgerAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &GrantLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantLedgerQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl GrantLedgerModifier for GrantLedgerQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &GrantLedger, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

/// Runs the grant ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantLedgerEngine {
    pub steps: Vec<Rc<dyn GrantLedgerModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantLedgerEngine {
    pub fn price(&mut self, item: &GrantLedger) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[GrantLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_grant_ledger_engine() -> GrantLedgerEngine {
    GrantLedgerEngine {
        steps: vec![
            Rc::new(RoundingGrantLedgerAdjuster { step: 1.0 }) as Rc<dyn GrantLedgerModifier>,
            Rc::new(MinimumGrantLedgerAdjuster { minimum: 10.9 }),
            Rc::new(ChannelGrantLedgerAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.75),
                    ("retail".to_string(), 1.39),
                    ("north".to_string(), 0.75),
                ]),
            }),
            Rc::new(GrantLedgerQuotaUnitsOffsetAdjuster { weight: 1.25 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 29715.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_ledger_engine();
        let item = new_grant_ledger("code-137", "label-586");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
