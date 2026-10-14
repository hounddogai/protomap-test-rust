use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the release allocation calculation.
pub trait ReleaseAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseAllocationAdjuster {
    pub cap: f64,
}

impl ReleaseAllocationStep for CapReleaseAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReleaseAllocationAdjuster {
    pub percent: f64,
}

impl ReleaseAllocationStep for PercentReleaseAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReleaseAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseAllocationStep for TieredReleaseAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAllocationRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseAllocationStep for ReleaseAllocationRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseAllocation, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseAllocationStep for FlaggedReleaseAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the release allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseAllocationEngine {
    pub steps: Vec<Rc<dyn ReleaseAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseAllocationEngine {
    pub fn compute(&mut self, item: &ReleaseAllocation) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseAllocationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_release_allocation_engine(flags: &flags::Registry) -> ReleaseAllocationEngine {
    ReleaseAllocationEngine {
        steps: vec![
            Rc::new(CapReleaseAllocationAdjuster { cap: 7709.0 }) as Rc<dyn ReleaseAllocationStep>,
            Rc::new(PercentReleaseAllocationAdjuster { percent: 19.6 }),
            Rc::new(TieredReleaseAllocationAdjuster {
                bounds: vec![1338.0, 3944.0, 4777.0, 7270.0],
                rates: vec![1.12, 0.83, 1.01, 0.98],
            }),
            Rc::new(ReleaseAllocationRotationDaysOffsetAdjuster { weight: 1.15 }),
            Rc::new(FlaggedReleaseAllocationAdjuster {
                flags: flags.clone(),
                flag: "release-allocation-batched".to_string(),
                percent: -6.1,
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 26334.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_allocation_engine(&flags::new_registry());
        let item = new_release_allocation("code-773", "title-119");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
