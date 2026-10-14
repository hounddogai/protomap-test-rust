use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the default runbook assignment calculation.
pub trait DefaultRunbookAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultRunbookAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultRunbookAssignmentAdjuster {
    pub percent: f64,
}

impl DefaultRunbookAssignmentAdjuster for PercentDefaultRunbookAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultRunbookAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultRunbookAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultRunbookAssignmentAdjuster for FlaggedDefaultRunbookAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultRunbookAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDefaultRunbookAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultRunbookAssignmentAdjuster for SegmentDefaultRunbookAssignmentAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &DefaultRunbookAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default runbook assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultRunbookAssignmentEngine {
    pub steps: Vec<Rc<dyn DefaultRunbookAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultRunbookAssignmentEngine {
    pub fn run(&mut self, item: &DefaultRunbookAssignment) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultRunbookAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultRunbookAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_runbook_assignment_engine(flags: &flags::Registry) -> DefaultRunbookAssignmentEngine {
    DefaultRunbookAssignmentEngine {
        steps: vec![
            Rc::new(PercentDefaultRunbookAssignmentAdjuster { percent: -16.0 })
                as Rc<dyn DefaultRunbookAssignmentAdjuster>,
            Rc::new(FlaggedDefaultRunbookAssignmentAdjuster {
                flags: flags.clone(),
                flag: "default-runbook-assignment-primary".to_string(),
                percent: -0.6,
            }),
            Rc::new(SegmentDefaultRunbookAssignmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.79)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 78111.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_runbook_assignment_engine(&flags::new_registry());
        let item = new_default_runbook_assignment("code-646", "name-962");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
