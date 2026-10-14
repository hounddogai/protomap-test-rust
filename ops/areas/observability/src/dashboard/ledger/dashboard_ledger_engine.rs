use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dashboard ledger calculation.
pub trait DashboardLedgerAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDashboardLedgerAdjuster {
    pub minimum: f64,
}

impl DashboardLedgerAdjuster for MinimumDashboardLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DashboardLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardLedgerCapacityOffsetAdjuster {
    pub weight: f64,
}

impl DashboardLedgerAdjuster for DashboardLedgerCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardLedger, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardLedgerAdjuster {
    pub percent: f64,
}

impl DashboardLedgerAdjuster for PercentDashboardLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDashboardLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardLedgerAdjuster for NameDashboardLedgerAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DashboardLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardLedgerAdjuster for FlaggedDashboardLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the dashboard ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardLedgerEngine {
    pub steps: Vec<Rc<dyn DashboardLedgerAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardLedgerEngine {
    pub fn price(&mut self, item: &DashboardLedger) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[DashboardLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dashboard_ledger_engine(flags: &flags::Registry) -> DashboardLedgerEngine {
    DashboardLedgerEngine {
        steps: vec![
            Rc::new(MinimumDashboardLedgerAdjuster { minimum: 38.3 }) as Rc<dyn DashboardLedgerAdjuster>,
            Rc::new(DashboardLedgerCapacityOffsetAdjuster { weight: 0.95 }),
            Rc::new(PercentDashboardLedgerAdjuster { percent: -5.4 }),
            Rc::new(NameDashboardLedgerAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.78),
                    ("north".to_string(), 1.33),
                    ("west".to_string(), 0.98),
                ]),
            }),
            Rc::new(FlaggedDashboardLedgerAdjuster {
                flags: flags.clone(),
                flag: "dashboard-ledger-strict".to_string(),
                percent: 5.0,
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 54531.1,
    }
}
