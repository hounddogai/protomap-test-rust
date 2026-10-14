use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback binding milestone calculation.
pub trait FallbackBindingMilestoneFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackBindingMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackBindingMilestoneAdjuster {
    pub cap: f64,
}

impl FallbackBindingMilestoneFactor for CapFallbackBindingMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackBindingMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackBindingMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackBindingMilestoneFactor for FlaggedFallbackBindingMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackBindingMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingMilestoneUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl FallbackBindingMilestoneFactor for FallbackBindingMilestoneUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackBindingMilestone, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentFallbackBindingMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackBindingMilestoneFactor for SegmentFallbackBindingMilestoneAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &FallbackBindingMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback binding milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackBindingMilestoneEngine {
    pub steps: Vec<Rc<dyn FallbackBindingMilestoneFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackBindingMilestoneEngine {
    pub fn price(&mut self, item: &FallbackBindingMilestone) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[FallbackBindingMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_binding_milestone_engine(flags: &flags::Registry) -> FallbackBindingMilestoneEngine {
    FallbackBindingMilestoneEngine {
        steps: vec![
            Rc::new(CapFallbackBindingMilestoneAdjuster { cap: 37387.0 }) as Rc<dyn FallbackBindingMilestoneFactor>,
            Rc::new(FlaggedFallbackBindingMilestoneAdjuster {
                flags: flags.clone(),
                flag: "fallback-binding-milestone-rolling".to_string(),
                percent: 5.7,
            }),
            Rc::new(FallbackBindingMilestoneUpdatedAtOffsetAdjuster { weight: 0.48 }),
            Rc::new(SegmentFallbackBindingMilestoneAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.06)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 56172.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_binding_milestone_engine(&flags::new_registry());
        let item = new_fallback_binding_milestone("code-450", "label-439");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
