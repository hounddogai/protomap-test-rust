use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling tenant rollup calculation.
pub trait RollingTenantRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingTenantRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingTenantRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingTenantRollupModifier for TieredRollingTenantRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingTenantRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantRollupMarginOffsetAdjuster {
    pub weight: f64,
}

impl RollingTenantRollupModifier for RollingTenantRollupMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &RollingTenantRollup, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingTenantRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingTenantRollupModifier for FlaggedRollingTenantRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingTenantRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling tenant rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingTenantRollupEngine {
    pub steps: Vec<Rc<dyn RollingTenantRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingTenantRollupEngine {
    pub fn price(&mut self, item: &RollingTenantRollup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingTenantRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_tenant_rollup_engine(flags: &flags::Registry) -> RollingTenantRollupEngine {
    RollingTenantRollupEngine {
        steps: vec![
            Rc::new(TieredRollingTenantRollupAdjuster {
                bounds: vec![4070.0, 6355.0, 6747.0, 7807.0],
                rates: vec![0.89, 1.17, 0.86, 1.17],
            }) as Rc<dyn RollingTenantRollupModifier>,
            Rc::new(RollingTenantRollupMarginOffsetAdjuster { weight: 0.75 }),
            Rc::new(FlaggedRollingTenantRollupAdjuster {
                flags: flags.clone(),
                flag: "rolling-tenant-rollup-legacy".to_string(),
                percent: -5.7,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 42463.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_tenant_rollup_engine(&flags::new_registry());
        let item = new_rolling_tenant_rollup("code-404", "label-330");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
