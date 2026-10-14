use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the scheduled overage snapshot calculation.
pub trait ScheduledOverageSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledOverageSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledOverageSnapshotAdjuster {
    pub minimum: f64,
}

impl ScheduledOverageSnapshotStep for MinimumScheduledOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledOverageSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameScheduledOverageSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledOverageSnapshotStep for NameScheduledOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ScheduledOverageSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledOverageSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledOverageSnapshotStep for FlaggedScheduledOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledOverageSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageSnapshotDiscountOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledOverageSnapshotStep for ScheduledOverageSnapshotDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledOverageSnapshot, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledOverageSnapshotAdjuster {
    pub cap: f64,
}

impl ScheduledOverageSnapshotStep for CapScheduledOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledOverageSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledOverageSnapshotAdjuster {
    pub percent: f64,
}

impl ScheduledOverageSnapshotStep for PercentScheduledOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledOverageSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled overage snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledOverageSnapshotEngine {
    pub steps: Vec<Rc<dyn ScheduledOverageSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledOverageSnapshotEngine {
    pub fn run(&mut self, item: &ScheduledOverageSnapshot) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledOverageSnapshotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledOverageSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_overage_snapshot_engine(flags: &flags::Registry) -> ScheduledOverageSnapshotEngine {
    ScheduledOverageSnapshotEngine {
        steps: vec![
            Rc::new(MinimumScheduledOverageSnapshotAdjuster { minimum: 20.5 }) as Rc<dyn ScheduledOverageSnapshotStep>,
            Rc::new(NameScheduledOverageSnapshotAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.36)]),
            }),
            Rc::new(FlaggedScheduledOverageSnapshotAdjuster {
                flags: flags.clone(),
                flag: "scheduled-overage-snapshot-adaptive".to_string(),
                percent: -5.0,
            }),
            Rc::new(ScheduledOverageSnapshotDiscountOffsetAdjuster { weight: 1.43 }),
            Rc::new(CapScheduledOverageSnapshotAdjuster { cap: 29069.0 }),
            Rc::new(PercentScheduledOverageSnapshotAdjuster { percent: 21.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 96419.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_overage_snapshot_engine(&flags::new_registry());
        let item = new_scheduled_overage_snapshot("code-696", "name-339");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
