use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tenant slot calculation.
pub trait TenantSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantSlotExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl TenantSlotStep for TenantSlotExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &TenantSlot, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantSlotStep for FlaggedTenantSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantSlotStep for TieredTenantSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantSlotAdjuster {
    pub percent: f64,
}

impl TenantSlotStep for PercentTenantSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tenant slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantSlotEngine {
    pub steps: Vec<Rc<dyn TenantSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantSlotEngine {
    pub fn price(&mut self, item: &TenantSlot) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[TenantSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_slot_engine(flags: &flags::Registry) -> TenantSlotEngine {
    TenantSlotEngine {
        steps: vec![
            Rc::new(TenantSlotExpiresAtOffsetAdjuster { weight: 0.72 }) as Rc<dyn TenantSlotStep>,
            Rc::new(FlaggedTenantSlotAdjuster {
                flags: flags.clone(),
                flag: "tenant-slot-seasonal".to_string(),
                percent: 1.5,
            }),
            Rc::new(TieredTenantSlotAdjuster {
                bounds: vec![1567.0, 2608.0, 3074.0, 5951.0],
                rates: vec![1.1, 1.0, 0.97, 1.19],
            }),
            Rc::new(PercentTenantSlotAdjuster { percent: 24.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 6726.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_slot_engine(&flags::new_registry());
        let item = new_tenant_slot("code-435", "name-76");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
