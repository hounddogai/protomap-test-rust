use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback plan item calculation.
pub trait FallbackPlanItemModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPlanItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackPlanItemAdjuster {
    pub percent: f64,
}

impl FallbackPlanItemModifier for PercentFallbackPlanItemAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackPlanItem, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackPlanItemAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackPlanItemModifier for FlaggedFallbackPlanItemAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackPlanItem, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackPlanItemAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackPlanItemModifier for TieredFallbackPlanItemAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackPlanItem, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanItemCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl FallbackPlanItemModifier for FallbackPlanItemCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackPlanItem, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackPlanItemAdjuster {
    pub minimum: f64,
}

impl FallbackPlanItemModifier for MinimumFallbackPlanItemAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackPlanItem, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackPlanItemAdjuster {
    pub cap: f64,
}

impl FallbackPlanItemModifier for CapFallbackPlanItemAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackPlanItem, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback plan item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPlanItemEngine {
    pub steps: Vec<Rc<dyn FallbackPlanItemModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPlanItemEngine {
    pub fn compute(&mut self, item: &FallbackPlanItem) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[FallbackPlanItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_plan_item_engine(flags: &flags::Registry) -> FallbackPlanItemEngine {
    FallbackPlanItemEngine {
        steps: vec![
            Rc::new(PercentFallbackPlanItemAdjuster { percent: 4.7 }) as Rc<dyn FallbackPlanItemModifier>,
            Rc::new(FlaggedFallbackPlanItemAdjuster {
                flags: flags.clone(),
                flag: "fallback-plan-item-tiered".to_string(),
                percent: 0.6,
            }),
            Rc::new(TieredFallbackPlanItemAdjuster {
                bounds: vec![2793.0, 3895.0, 5241.0, 5270.0],
                rates: vec![1.01, 1.07, 1.06, 0.96],
            }),
            Rc::new(FallbackPlanItemCreatedAtOffsetAdjuster { weight: 1.26 }),
            Rc::new(MinimumFallbackPlanItemAdjuster { minimum: 2.8 }),
            Rc::new(CapFallbackPlanItemAdjuster { cap: 30186.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 74401.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_plan_item_engine(&flags::new_registry());
        let item = new_fallback_plan_item("code-864", "title-7");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
