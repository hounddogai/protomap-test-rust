use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped data export policy calculation.
pub trait CappedDataExportPolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedDataExportPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedDataExportPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedDataExportPolicyFactor for TieredCappedDataExportPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedDataExportPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedDataExportPolicyAdjuster {
    pub minimum: f64,
}

impl CappedDataExportPolicyFactor for MinimumCappedDataExportPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedDataExportPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedDataExportPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedDataExportPolicyFactor for FlaggedCappedDataExportPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedDataExportPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedDataExportPolicyAdjuster {
    pub cap: f64,
}

impl CappedDataExportPolicyFactor for CapCappedDataExportPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedDataExportPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportPolicyQuantityOffsetAdjuster {
    pub weight: f64,
}

impl CappedDataExportPolicyFactor for CappedDataExportPolicyQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &CappedDataExportPolicy, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the capped data export policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedDataExportPolicyEngine {
    pub steps: Vec<Rc<dyn CappedDataExportPolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedDataExportPolicyEngine {
    pub fn compute(&mut self, item: &CappedDataExportPolicy) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedDataExportPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_data_export_policy_engine(flags: &flags::Registry) -> CappedDataExportPolicyEngine {
    CappedDataExportPolicyEngine {
        steps: vec![
            Rc::new(TieredCappedDataExportPolicyAdjuster {
                bounds: vec![1651.0, 2963.0],
                rates: vec![1.17, 1.15],
            }) as Rc<dyn CappedDataExportPolicyFactor>,
            Rc::new(MinimumCappedDataExportPolicyAdjuster { minimum: 36.9 }),
            Rc::new(FlaggedCappedDataExportPolicyAdjuster {
                flags: flags.clone(),
                flag: "capped-data-export-policy-scheduled".to_string(),
                percent: -1.0,
            }),
            Rc::new(CapCappedDataExportPolicyAdjuster { cap: 45710.0 }),
            Rc::new(CappedDataExportPolicyQuantityOffsetAdjuster { weight: 0.02 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 36173.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_data_export_policy_engine(&flags::new_registry());
        let item = new_capped_data_export_policy("code-231", "title-235");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
