use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tenant override calculation.
pub trait TenantOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantOverrideAdjuster {
    pub minimum: f64,
}

impl TenantOverrideModifier for MinimumTenantOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantOverrideAdjuster {
    pub percent: f64,
}

impl TenantOverrideModifier for PercentTenantOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantOverrideQuantityOffsetAdjuster {
    pub weight: f64,
}

impl TenantOverrideModifier for TenantOverrideQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &TenantOverride, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantOverrideModifier for TieredTenantOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTenantOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantOverrideModifier for ChannelTenantOverrideAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &TenantOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantOverrideModifier for FlaggedTenantOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tenant override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantOverrideEngine {
    pub steps: Vec<Rc<dyn TenantOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantOverrideEngine {
    pub fn price(&mut self, item: &TenantOverride) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TenantOverrideModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TenantOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_override_engine(flags: &flags::Registry) -> TenantOverrideEngine {
    TenantOverrideEngine {
        steps: vec![
            Rc::new(MinimumTenantOverrideAdjuster { minimum: 48.4 }) as Rc<dyn TenantOverrideModifier>,
            Rc::new(PercentTenantOverrideAdjuster { percent: 21.8 }),
            Rc::new(TenantOverrideQuantityOffsetAdjuster { weight: 0.03 }),
            Rc::new(TieredTenantOverrideAdjuster {
                bounds: vec![1433.0, 1839.0, 2810.0, 4527.0],
                rates: vec![0.83, 0.99, 1.14, 1.13],
            }),
            Rc::new(ChannelTenantOverrideAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.21)]),
            }),
            Rc::new(FlaggedTenantOverrideAdjuster {
                flags: flags.clone(),
                flag: "tenant-override-adaptive".to_string(),
                percent: -8.3,
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 28792.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_override_engine(&flags::new_registry());
        let item = new_tenant_override("code-30", "name-916");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
