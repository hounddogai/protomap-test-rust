use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan snapshot calculation.
pub trait PlanSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanSnapshotAdjuster {
    pub step: f64,
}

impl PlanSnapshotFactor for RoundingPlanSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanSnapshotFactor for FlaggedPlanSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanSnapshotAdjuster {
    pub percent: f64,
}

impl PlanSnapshotFactor for PercentPlanSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the plan snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanSnapshotEngine {
    pub steps: Vec<Rc<dyn PlanSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanSnapshotEngine {
    pub fn compute(&mut self, item: &PlanSnapshot) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanSnapshotFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_plan_snapshot_engine(flags: &flags::Registry) -> PlanSnapshotEngine {
    PlanSnapshotEngine {
        steps: vec![
            Rc::new(RoundingPlanSnapshotAdjuster { step: 0.5 }) as Rc<dyn PlanSnapshotFactor>,
            Rc::new(FlaggedPlanSnapshotAdjuster {
                flags: flags.clone(),
                flag: "plan-snapshot-primary".to_string(),
                percent: -1.6,
            }),
            Rc::new(PercentPlanSnapshotAdjuster { percent: 17.4 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 89114.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_snapshot_engine(&flags::new_registry());
        let item = new_plan_snapshot("code-609", "label-750");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
