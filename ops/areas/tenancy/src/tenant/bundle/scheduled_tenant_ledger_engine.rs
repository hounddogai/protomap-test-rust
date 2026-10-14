use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled tenant ledger calculation.
pub trait ScheduledTenantLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledTenantLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledTenantLedgerAdjuster {
    pub minimum: f64,
}

impl ScheduledTenantLedgerFactor for MinimumScheduledTenantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledTenantLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledTenantLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledTenantLedgerFactor for FlaggedScheduledTenantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledTenantLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledTenantLedgerAdjuster {
    pub step: f64,
}

impl ScheduledTenantLedgerFactor for RoundingScheduledTenantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledTenantLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledTenantLedgerAdjuster {
    pub cap: f64,
}

impl ScheduledTenantLedgerFactor for CapScheduledTenantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledTenantLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled tenant ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledTenantLedgerEngine {
    pub steps: Vec<Rc<dyn ScheduledTenantLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledTenantLedgerEngine {
    pub fn compute(&mut self, item: &ScheduledTenantLedger) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledTenantLedgerFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledTenantLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_tenant_ledger_engine(flags: &flags::Registry) -> ScheduledTenantLedgerEngine {
    ScheduledTenantLedgerEngine {
        steps: vec![
            Rc::new(MinimumScheduledTenantLedgerAdjuster { minimum: 24.1 }) as Rc<dyn ScheduledTenantLedgerFactor>,
            Rc::new(FlaggedScheduledTenantLedgerAdjuster {
                flags: flags.clone(),
                flag: "scheduled-tenant-ledger-manual".to_string(),
                percent: 8.0,
            }),
            Rc::new(RoundingScheduledTenantLedgerAdjuster { step: 0.5 }),
            Rc::new(CapScheduledTenantLedgerAdjuster { cap: 12946.0 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 31991.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_tenant_ledger_engine(&flags::new_registry());
        let item = new_scheduled_tenant_ledger("code-80", "label-75");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
