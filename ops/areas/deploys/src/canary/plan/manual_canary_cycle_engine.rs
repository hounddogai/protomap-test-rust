use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual canary cycle calculation.
pub trait ManualCanaryCycleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualCanaryCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualCanaryCycleAdjuster {
    pub step: f64,
}

impl ManualCanaryCycleModifier for RoundingManualCanaryCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualCanaryCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualCanaryCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualCanaryCycleModifier for FlaggedManualCanaryCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualCanaryCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualCanaryCycleAdjuster {
    pub cap: f64,
}

impl ManualCanaryCycleModifier for CapManualCanaryCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualCanaryCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameManualCanaryCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualCanaryCycleModifier for NameManualCanaryCycleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ManualCanaryCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualCanaryCycleAdjuster {
    pub percent: f64,
}

impl ManualCanaryCycleModifier for PercentManualCanaryCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualCanaryCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualCanaryCycleAdjuster {
    pub minimum: f64,
}

impl ManualCanaryCycleModifier for MinimumManualCanaryCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualCanaryCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual canary cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualCanaryCycleEngine {
    pub steps: Vec<Rc<dyn ManualCanaryCycleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualCanaryCycleEngine {
    pub fn compute(&mut self, item: &ManualCanaryCycle) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ManualCanaryCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_canary_cycle_engine(flags: &flags::Registry) -> ManualCanaryCycleEngine {
    ManualCanaryCycleEngine {
        steps: vec![
            Rc::new(RoundingManualCanaryCycleAdjuster { step: 0.1 }) as Rc<dyn ManualCanaryCycleModifier>,
            Rc::new(FlaggedManualCanaryCycleAdjuster {
                flags: flags.clone(),
                flag: "manual-canary-cycle-regional".to_string(),
                percent: 7.3,
            }),
            Rc::new(CapManualCanaryCycleAdjuster { cap: 20403.0 }),
            Rc::new(NameManualCanaryCycleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.31)]),
            }),
            Rc::new(PercentManualCanaryCycleAdjuster { percent: 11.3 }),
            Rc::new(MinimumManualCanaryCycleAdjuster { minimum: 20.2 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 85577.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_canary_cycle_engine(&flags::new_registry());
        let item = new_manual_canary_cycle("code-68", "name-391");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
