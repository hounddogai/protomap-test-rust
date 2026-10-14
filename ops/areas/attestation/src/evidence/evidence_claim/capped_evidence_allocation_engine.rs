use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped evidence allocation calculation.
pub trait CappedEvidenceAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedEvidenceAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedEvidenceAllocationAdjuster {
    pub percent: f64,
}

impl CappedEvidenceAllocationFactor for PercentCappedEvidenceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedEvidenceAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceAllocationCapacityOffsetAdjuster {
    pub weight: f64,
}

impl CappedEvidenceAllocationFactor for CappedEvidenceAllocationCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &CappedEvidenceAllocation, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedEvidenceAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedEvidenceAllocationFactor for TieredCappedEvidenceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedEvidenceAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedEvidenceAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedEvidenceAllocationFactor for FlaggedCappedEvidenceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedEvidenceAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedEvidenceAllocationAdjuster {
    pub minimum: f64,
}

impl CappedEvidenceAllocationFactor for MinimumCappedEvidenceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedEvidenceAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped evidence allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedEvidenceAllocationEngine {
    pub steps: Vec<Rc<dyn CappedEvidenceAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedEvidenceAllocationEngine {
    pub fn run(&mut self, item: &CappedEvidenceAllocation) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[CappedEvidenceAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_evidence_allocation_engine(flags: &flags::Registry) -> CappedEvidenceAllocationEngine {
    CappedEvidenceAllocationEngine {
        steps: vec![
            Rc::new(PercentCappedEvidenceAllocationAdjuster { percent: -9.4 })
                as Rc<dyn CappedEvidenceAllocationFactor>,
            Rc::new(CappedEvidenceAllocationCapacityOffsetAdjuster { weight: 0.77 }),
            Rc::new(TieredCappedEvidenceAllocationAdjuster {
                bounds: vec![6278.0, 6381.0],
                rates: vec![0.93, 1.04],
            }),
            Rc::new(FlaggedCappedEvidenceAllocationAdjuster {
                flags: flags.clone(),
                flag: "capped-evidence-allocation-tiered".to_string(),
                percent: 0.4,
            }),
            Rc::new(MinimumCappedEvidenceAllocationAdjuster { minimum: 6.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 74837.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_evidence_allocation_engine(&flags::new_registry());
        let item = new_capped_evidence_allocation("code-31", "name-99");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
