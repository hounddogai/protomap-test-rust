use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped region pin milestone calculation.
pub trait CappedRegionPinMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedRegionPinMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleCappedRegionPinMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedRegionPinMilestoneModifier for TitleCappedRegionPinMilestoneAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &CappedRegionPinMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedRegionPinMilestoneAdjuster {
    pub minimum: f64,
}

impl CappedRegionPinMilestoneModifier for MinimumCappedRegionPinMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedRegionPinMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedRegionPinMilestoneAdjuster {
    pub percent: f64,
}

impl CappedRegionPinMilestoneModifier for PercentCappedRegionPinMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedRegionPinMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped region pin milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedRegionPinMilestoneEngine {
    pub steps: Vec<Rc<dyn CappedRegionPinMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedRegionPinMilestoneEngine {
    pub fn price(&mut self, item: &CappedRegionPinMilestone) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[CappedRegionPinMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_region_pin_milestone_engine() -> CappedRegionPinMilestoneEngine {
    CappedRegionPinMilestoneEngine {
        steps: vec![
            Rc::new(TitleCappedRegionPinMilestoneAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.75)]),
            }) as Rc<dyn CappedRegionPinMilestoneModifier>,
            Rc::new(MinimumCappedRegionPinMilestoneAdjuster { minimum: 46.0 }),
            Rc::new(PercentCappedRegionPinMilestoneAdjuster { percent: 11.2 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 33127.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_region_pin_milestone_engine();
        let item = new_capped_region_pin_milestone("code-273", "title-261");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
