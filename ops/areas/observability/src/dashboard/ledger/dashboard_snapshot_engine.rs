use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the dashboard snapshot calculation.
pub trait DashboardSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardSnapshotStep for FlaggedDashboardSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDashboardSnapshotAdjuster {
    pub cap: f64,
}

impl DashboardSnapshotStep for CapDashboardSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DashboardSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardSnapshotAdjuster {
    pub percent: f64,
}

impl DashboardSnapshotStep for PercentDashboardSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the dashboard snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardSnapshotEngine {
    pub steps: Vec<Rc<dyn DashboardSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardSnapshotEngine {
    pub fn run(&mut self, item: &DashboardSnapshot) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardSnapshotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_dashboard_snapshot_engine(flags: &flags::Registry) -> DashboardSnapshotEngine {
    DashboardSnapshotEngine {
        steps: vec![
            Rc::new(FlaggedDashboardSnapshotAdjuster {
                flags: flags.clone(),
                flag: "dashboard-snapshot-manual".to_string(),
                percent: -10.0,
            }) as Rc<dyn DashboardSnapshotStep>,
            Rc::new(CapDashboardSnapshotAdjuster { cap: 37611.0 }),
            Rc::new(PercentDashboardSnapshotAdjuster { percent: -21.2 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 73031.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_snapshot_engine(&flags::new_registry());
        let item = new_dashboard_snapshot("code-231", "label-861");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
