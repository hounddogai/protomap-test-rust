use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback page ledger calculation.
pub trait FallbackPageLedgerAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPageLedger, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackPageLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackPageLedgerAdjuster for FlaggedFallbackPageLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackPageLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackPageLedgerAdjuster {
    pub cap: f64,
}

impl FallbackPageLedgerAdjuster for CapFallbackPageLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackPageLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackPageLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackPageLedgerAdjuster for TieredFallbackPageLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackPageLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackPageLedgerAdjuster {
    pub minimum: f64,
}

impl FallbackPageLedgerAdjuster for MinimumFallbackPageLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackPageLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback page ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPageLedgerEngine {
    pub steps: Vec<Rc<dyn FallbackPageLedgerAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPageLedgerEngine {
    pub fn evaluate(&mut self, item: &FallbackPageLedger) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FallbackPageLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_page_ledger_engine(flags: &flags::Registry) -> FallbackPageLedgerEngine {
    FallbackPageLedgerEngine {
        steps: vec![
            Rc::new(FlaggedFallbackPageLedgerAdjuster {
                flags: flags.clone(),
                flag: "fallback-page-ledger-capped".to_string(),
                percent: -2.4,
            }) as Rc<dyn FallbackPageLedgerAdjuster>,
            Rc::new(CapFallbackPageLedgerAdjuster { cap: 32363.0 }),
            Rc::new(TieredFallbackPageLedgerAdjuster {
                bounds: vec![3487.0, 9983.0],
                rates: vec![1.12, 1.12],
            }),
            Rc::new(MinimumFallbackPageLedgerAdjuster { minimum: 11.0 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 86179.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_page_ledger_engine(&flags::new_registry());
        let item = new_fallback_page_ledger("code-316", "name-23");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
