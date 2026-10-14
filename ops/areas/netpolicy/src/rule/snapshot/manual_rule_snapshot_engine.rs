use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual rule snapshot calculation.
pub trait ManualRuleSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualRuleSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualRuleSnapshotAdjuster {
    pub cap: f64,
}

impl ManualRuleSnapshotAdjuster for CapManualRuleSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualRuleSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameManualRuleSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualRuleSnapshotAdjuster for NameManualRuleSnapshotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ManualRuleSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualRuleSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualRuleSnapshotAdjuster for FlaggedManualRuleSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualRuleSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual rule snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualRuleSnapshotEngine {
    pub steps: Vec<Rc<dyn ManualRuleSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualRuleSnapshotEngine {
    pub fn run(&mut self, item: &ManualRuleSnapshot) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualRuleSnapshotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualRuleSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_rule_snapshot_engine(flags: &flags::Registry) -> ManualRuleSnapshotEngine {
    ManualRuleSnapshotEngine {
        steps: vec![
            Rc::new(CapManualRuleSnapshotAdjuster { cap: 7413.0 }) as Rc<dyn ManualRuleSnapshotAdjuster>,
            Rc::new(NameManualRuleSnapshotAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.0), ("retail".to_string(), 0.74)]),
            }),
            Rc::new(FlaggedManualRuleSnapshotAdjuster {
                flags: flags.clone(),
                flag: "manual-rule-snapshot-manual".to_string(),
                percent: 9.2,
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 91142.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_rule_snapshot_engine(&flags::new_registry());
        let item = new_manual_rule_snapshot("code-104", "name-104");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
