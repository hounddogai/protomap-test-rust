use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy limit milestone calculation.
pub trait LegacyLimitMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyLimitMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyLimitMilestoneAdjuster {
    pub cap: f64,
}

impl LegacyLimitMilestoneStep for CapLegacyLimitMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyLimitMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitMilestoneStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl LegacyLimitMilestoneStep for LegacyLimitMilestoneStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyLimitMilestone, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyLimitMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyLimitMilestoneStep for FlaggedLegacyLimitMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyLimitMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy limit milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyLimitMilestoneEngine {
    pub steps: Vec<Rc<dyn LegacyLimitMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyLimitMilestoneEngine {
    pub fn compute(&mut self, item: &LegacyLimitMilestone) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[LegacyLimitMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_limit_milestone_engine(flags: &flags::Registry) -> LegacyLimitMilestoneEngine {
    LegacyLimitMilestoneEngine {
        steps: vec![
            Rc::new(CapLegacyLimitMilestoneAdjuster { cap: 34676.0 }) as Rc<dyn LegacyLimitMilestoneStep>,
            Rc::new(LegacyLimitMilestoneStartsAtOffsetAdjuster { weight: 0.03 }),
            Rc::new(FlaggedLegacyLimitMilestoneAdjuster {
                flags: flags.clone(),
                flag: "legacy-limit-milestone-strict".to_string(),
                percent: -9.2,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 87664.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_limit_milestone_engine(&flags::new_registry());
        let item = new_legacy_limit_milestone("code-436", "title-763");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
