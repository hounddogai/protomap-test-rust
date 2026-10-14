use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dashboard manifest calculation.
pub trait DashboardManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionDashboardManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardManifestAdjuster for RegionDashboardManifestAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &DashboardManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardManifestAdjuster for TieredDashboardManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardManifestCapacityOffsetAdjuster {
    pub weight: f64,
}

impl DashboardManifestAdjuster for DashboardManifestCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardManifest, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDashboardManifestAdjuster {
    pub step: f64,
}

impl DashboardManifestAdjuster for RoundingDashboardManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DashboardManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardManifestAdjuster for FlaggedDashboardManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the dashboard manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardManifestEngine {
    pub steps: Vec<Rc<dyn DashboardManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardManifestEngine {
    pub fn compute(&mut self, item: &DashboardManifest) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[DashboardManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_dashboard_manifest_engine(flags: &flags::Registry) -> DashboardManifestEngine {
    DashboardManifestEngine {
        steps: vec![
            Rc::new(RegionDashboardManifestAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.84)]),
            }) as Rc<dyn DashboardManifestAdjuster>,
            Rc::new(TieredDashboardManifestAdjuster {
                bounds: vec![66.0, 2301.0, 7895.0],
                rates: vec![1.19, 0.87, 1.08],
            }),
            Rc::new(DashboardManifestCapacityOffsetAdjuster { weight: 1.34 }),
            Rc::new(RoundingDashboardManifestAdjuster { step: 5.0 }),
            Rc::new(FlaggedDashboardManifestAdjuster {
                flags: flags.clone(),
                flag: "dashboard-manifest-strict".to_string(),
                percent: -4.6,
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 38053.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_manifest_engine(&flags::new_registry());
        let item = new_dashboard_manifest("code-503", "label-624");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
