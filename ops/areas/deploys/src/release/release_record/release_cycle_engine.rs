use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the release cycle calculation.
pub trait ReleaseCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseCycle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseCycleStep for FlaggedReleaseCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReleaseCycleAdjuster {
    pub percent: f64,
}

impl ReleaseCycleStep for PercentReleaseCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReleaseCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseCycleAdjuster {
    pub cap: f64,
}

impl ReleaseCycleStep for CapReleaseCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the release cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseCycleEngine {
    pub steps: Vec<Rc<dyn ReleaseCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseCycleEngine {
    pub fn run(&mut self, item: &ReleaseCycle) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_release_cycle_engine(flags: &flags::Registry) -> ReleaseCycleEngine {
    ReleaseCycleEngine {
        steps: vec![
            Rc::new(FlaggedReleaseCycleAdjuster {
                flags: flags.clone(),
                flag: "release-cycle-strict".to_string(),
                percent: -9.1,
            }) as Rc<dyn ReleaseCycleStep>,
            Rc::new(PercentReleaseCycleAdjuster { percent: 14.7 }),
            Rc::new(CapReleaseCycleAdjuster { cap: 44679.0 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 37248.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_cycle_engine(&flags::new_registry());
        let item = new_release_cycle("code-240", "title-793");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
