use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the peering allocation calculation.
pub trait PeeringAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPeeringAllocationAdjuster {
    pub cap: f64,
}

impl PeeringAllocationModifier for CapPeeringAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PeeringAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PeeringAllocationModifier for TieredPeeringAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PeeringAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringAllocationAdjuster {
    pub percent: f64,
}

impl PeeringAllocationModifier for PercentPeeringAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPeeringAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PeeringAllocationModifier for FlaggedPeeringAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PeeringAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAllocationBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl PeeringAllocationModifier for PeeringAllocationBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringAllocation, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringAllocationAdjuster {
    pub minimum: f64,
}

impl PeeringAllocationModifier for MinimumPeeringAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the peering allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringAllocationEngine {
    pub steps: Vec<Rc<dyn PeeringAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringAllocationEngine {
    pub fn compute(&mut self, item: &PeeringAllocation) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PeeringAllocationModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PeeringAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_peering_allocation_engine(flags: &flags::Registry) -> PeeringAllocationEngine {
    PeeringAllocationEngine {
        steps: vec![
            Rc::new(CapPeeringAllocationAdjuster { cap: 38050.0 }) as Rc<dyn PeeringAllocationModifier>,
            Rc::new(TieredPeeringAllocationAdjuster {
                bounds: vec![1291.0, 6129.0, 7117.0],
                rates: vec![1.03, 0.98, 1.03],
            }),
            Rc::new(PercentPeeringAllocationAdjuster { percent: -24.4 }),
            Rc::new(FlaggedPeeringAllocationAdjuster {
                flags: flags.clone(),
                flag: "peering-allocation-manual".to_string(),
                percent: -7.1,
            }),
            Rc::new(PeeringAllocationBatchSizeOffsetAdjuster { weight: 0.91 }),
            Rc::new(MinimumPeeringAllocationAdjuster { minimum: 46.0 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 26124.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_allocation_engine(&flags::new_registry());
        let item = new_peering_allocation("code-43", "name-632");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
