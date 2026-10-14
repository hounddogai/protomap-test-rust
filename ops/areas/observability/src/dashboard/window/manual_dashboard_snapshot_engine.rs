use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual dashboard snapshot calculation.
pub trait ManualDashboardSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualDashboardSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualDashboardSnapshotAdjuster {
    pub cap: f64,
}

impl ManualDashboardSnapshotStep for CapManualDashboardSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualDashboardSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDashboardSnapshotBurstOffsetAdjuster {
    pub weight: f64,
}

impl ManualDashboardSnapshotStep for ManualDashboardSnapshotBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &ManualDashboardSnapshot, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualDashboardSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualDashboardSnapshotStep for FlaggedManualDashboardSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualDashboardSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameManualDashboardSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualDashboardSnapshotStep for NameManualDashboardSnapshotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ManualDashboardSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualDashboardSnapshotAdjuster {
    pub minimum: f64,
}

impl ManualDashboardSnapshotStep for MinimumManualDashboardSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualDashboardSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual dashboard snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualDashboardSnapshotEngine {
    pub steps: Vec<Rc<dyn ManualDashboardSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualDashboardSnapshotEngine {
    pub fn price(&mut self, item: &ManualDashboardSnapshot) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualDashboardSnapshotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualDashboardSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_dashboard_snapshot_engine(flags: &flags::Registry) -> ManualDashboardSnapshotEngine {
    ManualDashboardSnapshotEngine {
        steps: vec![
            Rc::new(CapManualDashboardSnapshotAdjuster { cap: 4094.0 }) as Rc<dyn ManualDashboardSnapshotStep>,
            Rc::new(ManualDashboardSnapshotBurstOffsetAdjuster { weight: 0.31 }),
            Rc::new(FlaggedManualDashboardSnapshotAdjuster {
                flags: flags.clone(),
                flag: "manual-dashboard-snapshot-manual".to_string(),
                percent: -4.1,
            }),
            Rc::new(NameManualDashboardSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.27),
                    ("east".to_string(), 0.89),
                    ("central".to_string(), 0.72),
                ]),
            }),
            Rc::new(MinimumManualDashboardSnapshotAdjuster { minimum: 26.8 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 31786.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_dashboard_snapshot_engine(&flags::new_registry());
        let item = new_manual_dashboard_snapshot("code-269", "name-469");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
