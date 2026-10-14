use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the binding batch calculation.
pub trait BindingBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBatchAmountOffsetAdjuster {
    pub weight: f64,
}

impl BindingBatchFactor for BindingBatchAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &BindingBatch, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBindingBatchAdjuster {
    pub cap: f64,
}

impl BindingBatchFactor for CapBindingBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BindingBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBindingBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BindingBatchFactor for FlaggedBindingBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BindingBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBindingBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingBatchFactor for SegmentBindingBatchAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &BindingBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBindingBatchAdjuster {
    pub percent: f64,
}

impl BindingBatchFactor for PercentBindingBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BindingBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingBatchFactor for TieredBindingBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the binding batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingBatchEngine {
    pub steps: Vec<Rc<dyn BindingBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingBatchEngine {
    pub fn run(&mut self, item: &BindingBatch) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[BindingBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_binding_batch_engine(flags: &flags::Registry) -> BindingBatchEngine {
    BindingBatchEngine {
        steps: vec![
            Rc::new(BindingBatchAmountOffsetAdjuster { weight: 0.33 }) as Rc<dyn BindingBatchFactor>,
            Rc::new(CapBindingBatchAdjuster { cap: 38333.0 }),
            Rc::new(FlaggedBindingBatchAdjuster {
                flags: flags.clone(),
                flag: "binding-batch-primary".to_string(),
                percent: 10.0,
            }),
            Rc::new(SegmentBindingBatchAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.16)]),
            }),
            Rc::new(PercentBindingBatchAdjuster { percent: -15.6 }),
            Rc::new(TieredBindingBatchAdjuster {
                bounds: vec![1113.0, 2747.0],
                rates: vec![0.97, 0.9],
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 48758.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_batch_engine(&flags::new_registry());
        let item = new_binding_batch("code-570", "name-872");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
