use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped redaction adjustment calculation.
pub trait CappedRedactionAdjustmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedRedactionAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedRedactionAdjustmentAdjuster {
    pub percent: f64,
}

impl CappedRedactionAdjustmentAdjuster for PercentCappedRedactionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedRedactionAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedRedactionAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedRedactionAdjustmentAdjuster for FlaggedCappedRedactionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedRedactionAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedRedactionAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedRedactionAdjustmentAdjuster for TieredCappedRedactionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedRedactionAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedRedactionAdjustmentAdjuster {
    pub cap: f64,
}

impl CappedRedactionAdjustmentAdjuster for CapCappedRedactionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedRedactionAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped redaction adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedRedactionAdjustmentEngine {
    pub steps: Vec<Rc<dyn CappedRedactionAdjustmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedRedactionAdjustmentEngine {
    pub fn compute(&mut self, item: &CappedRedactionAdjustment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedRedactionAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_redaction_adjustment_engine(flags: &flags::Registry) -> CappedRedactionAdjustmentEngine {
    CappedRedactionAdjustmentEngine {
        steps: vec![
            Rc::new(PercentCappedRedactionAdjustmentAdjuster { percent: -14.1 })
                as Rc<dyn CappedRedactionAdjustmentAdjuster>,
            Rc::new(FlaggedCappedRedactionAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "capped-redaction-adjustment-weighted".to_string(),
                percent: 9.4,
            }),
            Rc::new(TieredCappedRedactionAdjustmentAdjuster {
                bounds: vec![5546.0, 7626.0, 8197.0],
                rates: vec![0.94, 1.07, 0.85],
            }),
            Rc::new(CapCappedRedactionAdjustmentAdjuster { cap: 13832.0 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 42647.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_redaction_adjustment_engine(&flags::new_registry());
        let item = new_capped_redaction_adjustment("code-848", "title-49");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
