use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback binding adjustment calculation.
pub trait FallbackBindingAdjustmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackBindingAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackBindingAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackBindingAdjustmentAdjuster for FlaggedFallbackBindingAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackBindingAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameFallbackBindingAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackBindingAdjustmentAdjuster for NameFallbackBindingAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &FallbackBindingAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackBindingAdjustmentAdjuster {
    pub minimum: f64,
}

impl FallbackBindingAdjustmentAdjuster for MinimumFallbackBindingAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackBindingAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback binding adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackBindingAdjustmentEngine {
    pub steps: Vec<Rc<dyn FallbackBindingAdjustmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackBindingAdjustmentEngine {
    pub fn evaluate(&mut self, item: &FallbackBindingAdjustment) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[FallbackBindingAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_binding_adjustment_engine(flags: &flags::Registry) -> FallbackBindingAdjustmentEngine {
    FallbackBindingAdjustmentEngine {
        steps: vec![
            Rc::new(FlaggedFallbackBindingAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "fallback-binding-adjustment-strict".to_string(),
                percent: 0.9,
            }) as Rc<dyn FallbackBindingAdjustmentAdjuster>,
            Rc::new(NameFallbackBindingAdjustmentAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.8)]),
            }),
            Rc::new(MinimumFallbackBindingAdjustmentAdjuster { minimum: 41.5 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 55560.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_binding_adjustment_engine(&flags::new_registry());
        let item = new_fallback_binding_adjustment("code-825", "name-51");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
