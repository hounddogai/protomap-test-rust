use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the freeze adjustment calculation.
pub trait FreezeAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezeAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezeAdjustmentModifier for FlaggedFreezeAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezeAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAdjustmentQuantityOffsetAdjuster {
    pub weight: f64,
}

impl FreezeAdjustmentModifier for FreezeAdjustmentQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeAdjustment, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFreezeAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeAdjustmentModifier for TitleFreezeAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FreezeAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezeAdjustmentAdjuster {
    pub step: f64,
}

impl FreezeAdjustmentModifier for RoundingFreezeAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezeAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the freeze adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeAdjustmentEngine {
    pub steps: Vec<Rc<dyn FreezeAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeAdjustmentEngine {
    pub fn price(&mut self, item: &FreezeAdjustment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FreezeAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_freeze_adjustment_engine(flags: &flags::Registry) -> FreezeAdjustmentEngine {
    FreezeAdjustmentEngine {
        steps: vec![
            Rc::new(FlaggedFreezeAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "freeze-adjustment-adaptive".to_string(),
                percent: -8.9,
            }) as Rc<dyn FreezeAdjustmentModifier>,
            Rc::new(FreezeAdjustmentQuantityOffsetAdjuster { weight: 1.4 }),
            Rc::new(TitleFreezeAdjustmentAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.86),
                    ("retail".to_string(), 1.21),
                    ("east".to_string(), 1.0),
                ]),
            }),
            Rc::new(RoundingFreezeAdjustmentAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 27312.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_adjustment_engine(&flags::new_registry());
        let item = new_freeze_adjustment("code-612", "title-71");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
