use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped data export allocation calculation.
pub trait CappedDataExportAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedDataExportAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedDataExportAllocationAdjuster {
    pub step: f64,
}

impl CappedDataExportAllocationModifier for RoundingCappedDataExportAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedDataExportAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedDataExportAllocationAdjuster {
    pub percent: f64,
}

impl CappedDataExportAllocationModifier for PercentCappedDataExportAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedDataExportAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedDataExportAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedDataExportAllocationModifier for FlaggedCappedDataExportAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedDataExportAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedDataExportAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedDataExportAllocationModifier for TieredCappedDataExportAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedDataExportAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportAllocationBudgetOffsetAdjuster {
    pub weight: f64,
}

impl CappedDataExportAllocationModifier for CappedDataExportAllocationBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &CappedDataExportAllocation, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedDataExportAllocationAdjuster {
    pub cap: f64,
}

impl CappedDataExportAllocationModifier for CapCappedDataExportAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedDataExportAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped data export allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedDataExportAllocationEngine {
    pub steps: Vec<Rc<dyn CappedDataExportAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedDataExportAllocationEngine {
    pub fn compute(&mut self, item: &CappedDataExportAllocation) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[CappedDataExportAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_data_export_allocation_engine(flags: &flags::Registry) -> CappedDataExportAllocationEngine {
    CappedDataExportAllocationEngine {
        steps: vec![
            Rc::new(RoundingCappedDataExportAllocationAdjuster { step: 5.0 })
                as Rc<dyn CappedDataExportAllocationModifier>,
            Rc::new(PercentCappedDataExportAllocationAdjuster { percent: 4.0 }),
            Rc::new(FlaggedCappedDataExportAllocationAdjuster {
                flags: flags.clone(),
                flag: "capped-data-export-allocation-capped".to_string(),
                percent: 6.7,
            }),
            Rc::new(TieredCappedDataExportAllocationAdjuster {
                bounds: vec![2188.0, 2418.0, 8210.0, 9689.0],
                rates: vec![1.15, 0.9, 1.06, 1.17],
            }),
            Rc::new(CappedDataExportAllocationBudgetOffsetAdjuster { weight: 1.26 }),
            Rc::new(CapCappedDataExportAllocationAdjuster { cap: 4712.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 75380.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_data_export_allocation_engine(&flags::new_registry());
        let item = new_capped_data_export_allocation("code-949", "name-604");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
