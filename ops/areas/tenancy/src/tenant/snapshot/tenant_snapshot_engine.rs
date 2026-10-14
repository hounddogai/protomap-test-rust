use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tenant snapshot calculation.
pub trait TenantSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantSnapshotAdjuster for FlaggedTenantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantSnapshotAdjuster {
    pub cap: f64,
}

impl TenantSnapshotAdjuster for CapTenantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceTenantSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantSnapshotAdjuster for SourceTenantSnapshotAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &TenantSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantSnapshotAdjuster for TieredTenantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tenant snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantSnapshotEngine {
    pub steps: Vec<Rc<dyn TenantSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantSnapshotEngine {
    pub fn evaluate(&mut self, item: &TenantSnapshot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TenantSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tenant_snapshot_engine(flags: &flags::Registry) -> TenantSnapshotEngine {
    TenantSnapshotEngine {
        steps: vec![
            Rc::new(FlaggedTenantSnapshotAdjuster {
                flags: flags.clone(),
                flag: "tenant-snapshot-regional".to_string(),
                percent: -9.3,
            }) as Rc<dyn TenantSnapshotAdjuster>,
            Rc::new(CapTenantSnapshotAdjuster { cap: 32393.0 }),
            Rc::new(SourceTenantSnapshotAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.96), ("south".to_string(), 1.18)]),
            }),
            Rc::new(TieredTenantSnapshotAdjuster {
                bounds: vec![903.0, 5703.0],
                rates: vec![1.0, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 47343.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_snapshot_engine(&flags::new_registry());
        let item = new_tenant_snapshot("code-850", "name-442");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
