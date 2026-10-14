use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dashboard record calculation.
pub trait DashboardRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardRecordStep for TieredDashboardRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardRecordStep for FlaggedDashboardRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDashboardRecordAdjuster {
    pub step: f64,
}

impl DashboardRecordStep for RoundingDashboardRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DashboardRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRecordBudgetOffsetAdjuster {
    pub weight: f64,
}

impl DashboardRecordStep for DashboardRecordBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardRecord, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardRecordAdjuster {
    pub percent: f64,
}

impl DashboardRecordStep for PercentDashboardRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDashboardRecordAdjuster {
    pub cap: f64,
}

impl DashboardRecordStep for CapDashboardRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DashboardRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the dashboard record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardRecordEngine {
    pub steps: Vec<Rc<dyn DashboardRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardRecordEngine {
    pub fn run(&mut self, item: &DashboardRecord) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_dashboard_record_engine(flags: &flags::Registry) -> DashboardRecordEngine {
    DashboardRecordEngine {
        steps: vec![
            Rc::new(TieredDashboardRecordAdjuster {
                bounds: vec![5158.0, 8074.0],
                rates: vec![0.9, 0.99],
            }) as Rc<dyn DashboardRecordStep>,
            Rc::new(FlaggedDashboardRecordAdjuster {
                flags: flags.clone(),
                flag: "dashboard-record-pending".to_string(),
                percent: -6.6,
            }),
            Rc::new(RoundingDashboardRecordAdjuster { step: 0.05 }),
            Rc::new(DashboardRecordBudgetOffsetAdjuster { weight: 1.49 }),
            Rc::new(PercentDashboardRecordAdjuster { percent: 6.2 }),
            Rc::new(CapDashboardRecordAdjuster { cap: 27904.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 76812.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_record_engine(&flags::new_registry());
        let item = new_dashboard_record("code-236", "name-184");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
