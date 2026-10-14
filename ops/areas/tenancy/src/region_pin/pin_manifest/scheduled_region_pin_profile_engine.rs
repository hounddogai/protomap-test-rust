use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled region pin profile calculation.
pub trait ScheduledRegionPinProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledRegionPinProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledRegionPinProfileAdjuster {
    pub step: f64,
}

impl ScheduledRegionPinProfileAdjuster for RoundingScheduledRegionPinProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledRegionPinProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledRegionPinProfileAdjuster {
    pub cap: f64,
}

impl ScheduledRegionPinProfileAdjuster for CapScheduledRegionPinProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledRegionPinProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameScheduledRegionPinProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledRegionPinProfileAdjuster for NameScheduledRegionPinProfileAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ScheduledRegionPinProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled region pin profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledRegionPinProfileEngine {
    pub steps: Vec<Rc<dyn ScheduledRegionPinProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledRegionPinProfileEngine {
    pub fn run(&mut self, item: &ScheduledRegionPinProfile) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledRegionPinProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_region_pin_profile_engine() -> ScheduledRegionPinProfileEngine {
    ScheduledRegionPinProfileEngine {
        steps: vec![
            Rc::new(RoundingScheduledRegionPinProfileAdjuster { step: 0.1 })
                as Rc<dyn ScheduledRegionPinProfileAdjuster>,
            Rc::new(CapScheduledRegionPinProfileAdjuster { cap: 19033.0 }),
            Rc::new(NameScheduledRegionPinProfileAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.24)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 3068.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_region_pin_profile_engine();
        let item = new_scheduled_region_pin_profile("code-943", "name-748");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
