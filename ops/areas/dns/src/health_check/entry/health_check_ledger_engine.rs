use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the health check ledger calculation.
pub trait HealthCheckLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckLedgerDiscountOffsetAdjuster {
    pub weight: f64,
}

impl HealthCheckLedgerStep for HealthCheckLedgerDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &HealthCheckLedger, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckLedgerAdjuster {
    pub minimum: f64,
}

impl HealthCheckLedgerStep for MinimumHealthCheckLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckLedgerStep for FlaggedHealthCheckLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentHealthCheckLedgerAdjuster {
    pub percent: f64,
}

impl HealthCheckLedgerStep for PercentHealthCheckLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &HealthCheckLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the health check ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckLedgerEngine {
    pub steps: Vec<Rc<dyn HealthCheckLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckLedgerEngine {
    pub fn price(&mut self, item: &HealthCheckLedger) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_health_check_ledger_engine(flags: &flags::Registry) -> HealthCheckLedgerEngine {
    HealthCheckLedgerEngine {
        steps: vec![
            Rc::new(HealthCheckLedgerDiscountOffsetAdjuster { weight: 1.81 }) as Rc<dyn HealthCheckLedgerStep>,
            Rc::new(MinimumHealthCheckLedgerAdjuster { minimum: 33.0 }),
            Rc::new(FlaggedHealthCheckLedgerAdjuster {
                flags: flags.clone(),
                flag: "health-check-ledger-adaptive".to_string(),
                percent: 9.9,
            }),
            Rc::new(PercentHealthCheckLedgerAdjuster { percent: 10.3 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 21995.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_ledger_engine(&flags::new_registry());
        let item = new_health_check_ledger("code-686", "title-938");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
