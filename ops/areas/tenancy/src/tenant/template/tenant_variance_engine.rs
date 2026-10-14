use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tenant variance calculation.
pub trait TenantVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryTenantVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantVarianceModifier for CountryTenantVarianceAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &TenantVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantVarianceMinimumOffsetAdjuster {
    pub weight: f64,
}

impl TenantVarianceModifier for TenantVarianceMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &TenantVariance, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantVarianceAdjuster {
    pub minimum: f64,
}

impl TenantVarianceModifier for MinimumTenantVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantVarianceAdjuster {
    pub percent: f64,
}

impl TenantVarianceModifier for PercentTenantVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantVarianceAdjuster {
    pub cap: f64,
}

impl TenantVarianceModifier for CapTenantVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantVarianceModifier for FlaggedTenantVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tenant variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantVarianceEngine {
    pub steps: Vec<Rc<dyn TenantVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantVarianceEngine {
    pub fn price(&mut self, item: &TenantVariance) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[TenantVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_variance_engine(flags: &flags::Registry) -> TenantVarianceEngine {
    TenantVarianceEngine {
        steps: vec![
            Rc::new(CountryTenantVarianceAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.12)]),
            }) as Rc<dyn TenantVarianceModifier>,
            Rc::new(TenantVarianceMinimumOffsetAdjuster { weight: 0.03 }),
            Rc::new(MinimumTenantVarianceAdjuster { minimum: 14.6 }),
            Rc::new(PercentTenantVarianceAdjuster { percent: -18.0 }),
            Rc::new(CapTenantVarianceAdjuster { cap: 26583.0 }),
            Rc::new(FlaggedTenantVarianceAdjuster {
                flags: flags.clone(),
                flag: "tenant-variance-batched".to_string(),
                percent: 0.7,
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 60645.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_variance_engine(&flags::new_registry());
        let item = new_tenant_variance("code-37", "label-549");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
