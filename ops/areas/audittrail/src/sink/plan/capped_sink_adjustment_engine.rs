use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the capped sink adjustment calculation.
pub trait CappedSinkAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedSinkAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedSinkAdjustmentAdjuster {
    pub minimum: f64,
}

impl CappedSinkAdjustmentFactor for MinimumCappedSinkAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedSinkAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkAdjustmentMinimumOffsetAdjuster {
    pub weight: f64,
}

impl CappedSinkAdjustmentFactor for CappedSinkAdjustmentMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &CappedSinkAdjustment, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedSinkAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedSinkAdjustmentFactor for FlaggedCappedSinkAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedSinkAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedSinkAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedSinkAdjustmentFactor for TieredCappedSinkAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedSinkAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedSinkAdjustmentAdjuster {
    pub percent: f64,
}

impl CappedSinkAdjustmentFactor for PercentCappedSinkAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedSinkAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped sink adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedSinkAdjustmentEngine {
    pub steps: Vec<Rc<dyn CappedSinkAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedSinkAdjustmentEngine {
    pub fn evaluate(&mut self, item: &CappedSinkAdjustment) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedSinkAdjustmentFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[CappedSinkAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_sink_adjustment_engine(flags: &flags::Registry) -> CappedSinkAdjustmentEngine {
    CappedSinkAdjustmentEngine {
        steps: vec![
            Rc::new(MinimumCappedSinkAdjustmentAdjuster { minimum: 13.1 }) as Rc<dyn CappedSinkAdjustmentFactor>,
            Rc::new(CappedSinkAdjustmentMinimumOffsetAdjuster { weight: 1.39 }),
            Rc::new(FlaggedCappedSinkAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "capped-sink-adjustment-default".to_string(),
                percent: -5.9,
            }),
            Rc::new(TieredCappedSinkAdjustmentAdjuster {
                bounds: vec![520.0, 1017.0],
                rates: vec![0.81, 0.99],
            }),
            Rc::new(PercentCappedSinkAdjustmentAdjuster { percent: 4.4 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 86349.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_sink_adjustment_engine(&flags::new_registry());
        let item = new_capped_sink_adjustment("code-377", "label-151");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
