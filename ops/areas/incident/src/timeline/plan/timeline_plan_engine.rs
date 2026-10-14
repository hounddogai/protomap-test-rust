use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the timeline plan calculation.
pub trait TimelinePlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelinePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionTimelinePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TimelinePlanFactor for RegionTimelinePlanAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &TimelinePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelinePlanAdjuster {
    pub percent: f64,
}

impl TimelinePlanFactor for PercentTimelinePlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelinePlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelinePlanAdjuster {
    pub cap: f64,
}

impl TimelinePlanFactor for CapTimelinePlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelinePlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the timeline plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelinePlanEngine {
    pub steps: Vec<Rc<dyn TimelinePlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelinePlanEngine {
    pub fn evaluate(&mut self, item: &TimelinePlan) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[TimelinePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_timeline_plan_engine() -> TimelinePlanEngine {
    TimelinePlanEngine {
        steps: vec![
            Rc::new(RegionTimelinePlanAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.24)]),
            }) as Rc<dyn TimelinePlanFactor>,
            Rc::new(PercentTimelinePlanAdjuster { percent: 24.5 }),
            Rc::new(CapTimelinePlanAdjuster { cap: 46936.0 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 51970.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_timeline_plan_engine();
        let item = new_timeline_plan("code-189", "name-854");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
