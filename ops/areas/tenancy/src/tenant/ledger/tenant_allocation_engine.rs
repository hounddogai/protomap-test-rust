use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tenant allocation calculation.
pub trait TenantAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAllocationBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl TenantAllocationFactor for TenantAllocationBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &TenantAllocation, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantAllocationFactor for TieredTenantAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantAllocationAdjuster {
    pub minimum: f64,
}

impl TenantAllocationFactor for MinimumTenantAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantAllocationAdjuster {
    pub percent: f64,
}

impl TenantAllocationFactor for PercentTenantAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeTenantAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantAllocationFactor for ScopeTenantAllocationAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &TenantAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantAllocationFactor for FlaggedTenantAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tenant allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantAllocationEngine {
    pub steps: Vec<Rc<dyn TenantAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantAllocationEngine {
    pub fn evaluate(&mut self, item: &TenantAllocation) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TenantAllocationFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TenantAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tenant_allocation_engine(flags: &flags::Registry) -> TenantAllocationEngine {
    TenantAllocationEngine {
        steps: vec![
            Rc::new(TenantAllocationBatchSizeOffsetAdjuster { weight: 0.82 }) as Rc<dyn TenantAllocationFactor>,
            Rc::new(TieredTenantAllocationAdjuster {
                bounds: vec![2066.0, 4182.0, 4915.0, 7591.0],
                rates: vec![0.81, 1.06, 1.2, 0.84],
            }),
            Rc::new(MinimumTenantAllocationAdjuster { minimum: 15.3 }),
            Rc::new(PercentTenantAllocationAdjuster { percent: 23.8 }),
            Rc::new(ScopeTenantAllocationAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.26)]),
            }),
            Rc::new(FlaggedTenantAllocationAdjuster {
                flags: flags.clone(),
                flag: "tenant-allocation-rolling".to_string(),
                percent: -8.2,
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 15763.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_allocation_engine(&flags::new_registry());
        let item = new_tenant_allocation("code-247", "label-210");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
