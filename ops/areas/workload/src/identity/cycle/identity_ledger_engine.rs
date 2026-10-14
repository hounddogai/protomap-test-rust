use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the identity ledger calculation.
pub trait IdentityLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameIdentityLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityLedgerFactor for NameIdentityLedgerAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &IdentityLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIdentityLedgerAdjuster {
    pub minimum: f64,
}

impl IdentityLedgerFactor for MinimumIdentityLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IdentityLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityLedgerFactor for FlaggedIdentityLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityLedgerFactor for TieredIdentityLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the identity ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityLedgerEngine {
    pub steps: Vec<Rc<dyn IdentityLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityLedgerEngine {
    pub fn compute(&mut self, item: &IdentityLedger) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentityLedgerFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentityLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_identity_ledger_engine(flags: &flags::Registry) -> IdentityLedgerEngine {
    IdentityLedgerEngine {
        steps: vec![
            Rc::new(NameIdentityLedgerAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.97), ("south".to_string(), 0.92)]),
            }) as Rc<dyn IdentityLedgerFactor>,
            Rc::new(MinimumIdentityLedgerAdjuster { minimum: 45.3 }),
            Rc::new(FlaggedIdentityLedgerAdjuster {
                flags: flags.clone(),
                flag: "identity-ledger-batched".to_string(),
                percent: -7.9,
            }),
            Rc::new(TieredIdentityLedgerAdjuster {
                bounds: vec![1657.0, 4403.0, 5099.0, 9128.0],
                rates: vec![0.92, 1.13, 1.09, 1.06],
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 65388.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_ledger_engine(&flags::new_registry());
        let item = new_identity_ledger("code-340", "name-258");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
