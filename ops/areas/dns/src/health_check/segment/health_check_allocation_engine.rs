use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the health check allocation calculation.
pub trait HealthCheckAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingHealthCheckAllocationAdjuster {
    pub step: f64,
}

impl HealthCheckAllocationAdjuster for RoundingHealthCheckAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &HealthCheckAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckAllocationAdjuster for FlaggedHealthCheckAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapHealthCheckAllocationAdjuster {
    pub cap: f64,
}

impl HealthCheckAllocationAdjuster for CapHealthCheckAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &HealthCheckAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameHealthCheckAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckAllocationAdjuster for NameHealthCheckAllocationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &HealthCheckAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAllocationVersionOffsetAdjuster {
    pub weight: f64,
}

impl HealthCheckAllocationAdjuster for HealthCheckAllocationVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &HealthCheckAllocation, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckAllocationAdjuster for TieredHealthCheckAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the health check allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckAllocationEngine {
    pub steps: Vec<Rc<dyn HealthCheckAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckAllocationEngine {
    pub fn price(&mut self, item: &HealthCheckAllocation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_health_check_allocation_engine(flags: &flags::Registry) -> HealthCheckAllocationEngine {
    HealthCheckAllocationEngine {
        steps: vec![
            Rc::new(RoundingHealthCheckAllocationAdjuster { step: 5.0 }) as Rc<dyn HealthCheckAllocationAdjuster>,
            Rc::new(FlaggedHealthCheckAllocationAdjuster {
                flags: flags.clone(),
                flag: "health-check-allocation-manual".to_string(),
                percent: -5.3,
            }),
            Rc::new(CapHealthCheckAllocationAdjuster { cap: 18425.0 }),
            Rc::new(NameHealthCheckAllocationAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.71),
                    ("west".to_string(), 1.08),
                    ("south".to_string(), 0.75),
                ]),
            }),
            Rc::new(HealthCheckAllocationVersionOffsetAdjuster { weight: 1.03 }),
            Rc::new(TieredHealthCheckAllocationAdjuster {
                bounds: vec![141.0, 2936.0, 5539.0, 6797.0],
                rates: vec![1.0, 0.93, 1.07, 0.8],
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 23414.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_allocation_engine(&flags::new_registry());
        let item = new_health_check_allocation("code-405", "name-402");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
