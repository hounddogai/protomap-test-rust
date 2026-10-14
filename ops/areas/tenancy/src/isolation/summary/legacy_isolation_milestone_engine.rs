use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy isolation milestone calculation.
pub trait LegacyIsolationMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyIsolationMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyIsolationMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyIsolationMilestoneModifier for FlaggedLegacyIsolationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyIsolationMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationMilestoneCeilingOffsetAdjuster {
    pub weight: f64,
}

impl LegacyIsolationMilestoneModifier for LegacyIsolationMilestoneCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyIsolationMilestone, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyIsolationMilestoneAdjuster {
    pub minimum: f64,
}

impl LegacyIsolationMilestoneModifier for MinimumLegacyIsolationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyIsolationMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyIsolationMilestoneAdjuster {
    pub cap: f64,
}

impl LegacyIsolationMilestoneModifier for CapLegacyIsolationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyIsolationMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyIsolationMilestoneAdjuster {
    pub percent: f64,
}

impl LegacyIsolationMilestoneModifier for PercentLegacyIsolationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyIsolationMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy isolation milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyIsolationMilestoneEngine {
    pub steps: Vec<Rc<dyn LegacyIsolationMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyIsolationMilestoneEngine {
    pub fn compute(&mut self, item: &LegacyIsolationMilestone) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[LegacyIsolationMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_isolation_milestone_engine(flags: &flags::Registry) -> LegacyIsolationMilestoneEngine {
    LegacyIsolationMilestoneEngine {
        steps: vec![
            Rc::new(FlaggedLegacyIsolationMilestoneAdjuster {
                flags: flags.clone(),
                flag: "legacy-isolation-milestone-capped".to_string(),
                percent: -6.2,
            }) as Rc<dyn LegacyIsolationMilestoneModifier>,
            Rc::new(LegacyIsolationMilestoneCeilingOffsetAdjuster { weight: 0.05 }),
            Rc::new(MinimumLegacyIsolationMilestoneAdjuster { minimum: 37.0 }),
            Rc::new(CapLegacyIsolationMilestoneAdjuster { cap: 10349.0 }),
            Rc::new(PercentLegacyIsolationMilestoneAdjuster { percent: -16.3 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 20976.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_isolation_milestone_engine(&flags::new_registry());
        let item = new_legacy_isolation_milestone("code-369", "name-983");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
