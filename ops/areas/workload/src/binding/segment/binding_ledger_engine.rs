use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the binding ledger calculation.
pub trait BindingLedgerModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingLedgerModifier for TieredBindingLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionBindingLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingLedgerModifier for RegionBindingLedgerAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &BindingLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBindingLedgerAdjuster {
    pub cap: f64,
}

impl BindingLedgerModifier for CapBindingLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BindingLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the binding ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingLedgerEngine {
    pub steps: Vec<Rc<dyn BindingLedgerModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingLedgerEngine {
    pub fn run(&mut self, item: &BindingLedger) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BindingLedgerModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BindingLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_binding_ledger_engine() -> BindingLedgerEngine {
    BindingLedgerEngine {
        steps: vec![
            Rc::new(TieredBindingLedgerAdjuster {
                bounds: vec![3318.0, 9538.0],
                rates: vec![0.83, 0.85],
            }) as Rc<dyn BindingLedgerModifier>,
            Rc::new(RegionBindingLedgerAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.9), ("online".to_string(), 1.25)]),
            }),
            Rc::new(CapBindingLedgerAdjuster { cap: 32600.0 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 59481.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_ledger_engine();
        let item = new_binding_ledger("code-526", "label-670");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
