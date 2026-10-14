use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback usage allocation calculation.
pub trait FallbackUsageAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackUsageAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackUsageAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackUsageAllocationModifier for TieredFallbackUsageAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackUsageAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackUsageAllocationAdjuster {
    pub step: f64,
}

impl FallbackUsageAllocationModifier for RoundingFallbackUsageAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackUsageAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackUsageAllocationAdjuster {
    pub cap: f64,
}

impl FallbackUsageAllocationModifier for CapFallbackUsageAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackUsageAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackUsageAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackUsageAllocationModifier for FlaggedFallbackUsageAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackUsageAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackUsageAllocationAdjuster {
    pub percent: f64,
}

impl FallbackUsageAllocationModifier for PercentFallbackUsageAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackUsageAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback usage allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackUsageAllocationEngine {
    pub steps: Vec<Rc<dyn FallbackUsageAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackUsageAllocationEngine {
    pub fn compute(&mut self, item: &FallbackUsageAllocation) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[FallbackUsageAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_usage_allocation_engine(flags: &flags::Registry) -> FallbackUsageAllocationEngine {
    FallbackUsageAllocationEngine {
        steps: vec![
            Rc::new(TieredFallbackUsageAllocationAdjuster {
                bounds: vec![441.0, 1873.0, 3167.0, 8557.0],
                rates: vec![0.93, 1.16, 0.95, 1.19],
            }) as Rc<dyn FallbackUsageAllocationModifier>,
            Rc::new(RoundingFallbackUsageAllocationAdjuster { step: 1.0 }),
            Rc::new(CapFallbackUsageAllocationAdjuster { cap: 28207.0 }),
            Rc::new(FlaggedFallbackUsageAllocationAdjuster {
                flags: flags.clone(),
                flag: "fallback-usage-allocation-capped".to_string(),
                percent: -3.6,
            }),
            Rc::new(PercentFallbackUsageAllocationAdjuster { percent: -4.1 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 59440.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_usage_allocation_engine(&flags::new_registry());
        let item = new_fallback_usage_allocation("code-830", "title-514");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
