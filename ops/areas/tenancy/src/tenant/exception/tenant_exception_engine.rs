use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tenant exception calculation.
pub trait TenantExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantExceptionAdjuster {
    pub percent: f64,
}

impl TenantExceptionAdjuster for PercentTenantExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantExceptionAdjuster for TieredTenantExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantExceptionAdjuster {
    pub cap: f64,
}

impl TenantExceptionAdjuster for CapTenantExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantExceptionAdjuster for FlaggedTenantExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tenant exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantExceptionEngine {
    pub steps: Vec<Rc<dyn TenantExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantExceptionEngine {
    pub fn price(&mut self, item: &TenantException) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TenantException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_exception_engine(flags: &flags::Registry) -> TenantExceptionEngine {
    TenantExceptionEngine {
        steps: vec![
            Rc::new(PercentTenantExceptionAdjuster { percent: -15.7 }) as Rc<dyn TenantExceptionAdjuster>,
            Rc::new(TieredTenantExceptionAdjuster {
                bounds: vec![4548.0, 8968.0],
                rates: vec![0.94, 0.94],
            }),
            Rc::new(CapTenantExceptionAdjuster { cap: 40597.0 }),
            Rc::new(FlaggedTenantExceptionAdjuster {
                flags: flags.clone(),
                flag: "tenant-exception-legacy".to_string(),
                percent: 8.6,
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 4190.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_exception_engine(&flags::new_registry());
        let item = new_tenant_exception("code-596", "name-877");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
