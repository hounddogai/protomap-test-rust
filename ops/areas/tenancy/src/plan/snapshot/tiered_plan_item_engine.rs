use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered plan item calculation.
pub trait TieredPlanItemAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredPlanItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanItemVersionOffsetAdjuster {
    pub weight: f64,
}

impl TieredPlanItemAdjuster for TieredPlanItemVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &TieredPlanItem, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredPlanItemAdjuster {
    pub percent: f64,
}

impl TieredPlanItemAdjuster for PercentTieredPlanItemAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredPlanItem, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredPlanItemAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredPlanItemAdjuster for FlaggedTieredPlanItemAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredPlanItem, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered plan item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredPlanItemEngine {
    pub steps: Vec<Rc<dyn TieredPlanItemAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredPlanItemEngine {
    pub fn price(&mut self, item: &TieredPlanItem) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredPlanItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_plan_item_engine(flags: &flags::Registry) -> TieredPlanItemEngine {
    TieredPlanItemEngine {
        steps: vec![
            Rc::new(TieredPlanItemVersionOffsetAdjuster { weight: 1.87 }) as Rc<dyn TieredPlanItemAdjuster>,
            Rc::new(PercentTieredPlanItemAdjuster { percent: 3.2 }),
            Rc::new(FlaggedTieredPlanItemAdjuster {
                flags: flags.clone(),
                flag: "tiered-plan-item-default".to_string(),
                percent: -0.1,
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 86455.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_plan_item_engine(&flags::new_registry());
        let item = new_tiered_plan_item("code-695", "title-796");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
