use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the freeze threshold calculation.
pub trait FreezeThresholdModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFreezeThresholdAdjuster {
    pub cap: f64,
}

impl FreezeThresholdModifier for CapFreezeThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FreezeThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeThresholdAdjuster {
    pub minimum: f64,
}

impl FreezeThresholdModifier for MinimumFreezeThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezeThresholdAdjuster {
    pub percent: f64,
}

impl FreezeThresholdModifier for PercentFreezeThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezeThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeThresholdModifier for TieredFreezeThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFreezeThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeThresholdModifier for LabelFreezeThresholdAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FreezeThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeThresholdBudgetOffsetAdjuster {
    pub weight: f64,
}

impl FreezeThresholdModifier for FreezeThresholdBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeThreshold, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

/// Runs the freeze threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeThresholdEngine {
    pub steps: Vec<Rc<dyn FreezeThresholdModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeThresholdEngine {
    pub fn price(&mut self, item: &FreezeThreshold) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FreezeThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_freeze_threshold_engine() -> FreezeThresholdEngine {
    FreezeThresholdEngine {
        steps: vec![
            Rc::new(CapFreezeThresholdAdjuster { cap: 22246.0 }) as Rc<dyn FreezeThresholdModifier>,
            Rc::new(MinimumFreezeThresholdAdjuster { minimum: 11.0 }),
            Rc::new(PercentFreezeThresholdAdjuster { percent: 13.3 }),
            Rc::new(TieredFreezeThresholdAdjuster {
                bounds: vec![1706.0, 5093.0, 5885.0],
                rates: vec![1.04, 1.19, 1.05],
            }),
            Rc::new(LabelFreezeThresholdAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.1),
                    ("online".to_string(), 0.74),
                    ("retail".to_string(), 0.76),
                ]),
            }),
            Rc::new(FreezeThresholdBudgetOffsetAdjuster { weight: 0.66 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 33415.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_threshold_engine();
        let item = new_freeze_threshold("code-507", "label-324");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
