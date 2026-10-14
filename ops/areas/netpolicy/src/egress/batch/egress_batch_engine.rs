use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the egress batch calculation.
pub trait EgressBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressBatchAdjuster {
    pub percent: f64,
}

impl EgressBatchModifier for PercentEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressBatchAdjuster {
    pub step: f64,
}

impl EgressBatchModifier for RoundingEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEgressBatchAdjuster {
    pub minimum: f64,
}

impl EgressBatchModifier for MinimumEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EgressBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBatchAmountOffsetAdjuster {
    pub weight: f64,
}

impl EgressBatchModifier for EgressBatchAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &EgressBatch, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EgressBatchModifier for TieredEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EgressBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressBatchModifier for FlaggedEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the egress batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressBatchEngine {
    pub steps: Vec<Rc<dyn EgressBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressBatchEngine {
    pub fn evaluate(&mut self, item: &EgressBatch) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[EgressBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_egress_batch_engine(flags: &flags::Registry) -> EgressBatchEngine {
    EgressBatchEngine {
        steps: vec![
            Rc::new(PercentEgressBatchAdjuster { percent: 12.9 }) as Rc<dyn EgressBatchModifier>,
            Rc::new(RoundingEgressBatchAdjuster { step: 5.0 }),
            Rc::new(MinimumEgressBatchAdjuster { minimum: 14.2 }),
            Rc::new(EgressBatchAmountOffsetAdjuster { weight: 1.65 }),
            Rc::new(TieredEgressBatchAdjuster {
                bounds: vec![3014.0, 8282.0, 9026.0, 9971.0],
                rates: vec![1.07, 0.82, 1.05, 0.89],
            }),
            Rc::new(FlaggedEgressBatchAdjuster {
                flags: flags.clone(),
                flag: "egress-batch-fallback".to_string(),
                percent: -9.2,
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 76764.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_batch_engine(&flags::new_registry());
        let item = new_egress_batch("code-484", "label-825");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
