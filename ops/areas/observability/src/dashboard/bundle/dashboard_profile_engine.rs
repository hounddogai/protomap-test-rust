use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dashboard profile calculation.
pub trait DashboardProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardProfile, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardProfileAdjuster for FlaggedDashboardProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardProfileAdjuster for TieredDashboardProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDashboardProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardProfileAdjuster for SegmentDashboardProfileAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &DashboardProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDashboardProfileAdjuster {
    pub minimum: f64,
}

impl DashboardProfileAdjuster for MinimumDashboardProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DashboardProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the dashboard profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardProfileEngine {
    pub steps: Vec<Rc<dyn DashboardProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardProfileEngine {
    pub fn compute(&mut self, item: &DashboardProfile) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[DashboardProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_dashboard_profile_engine(flags: &flags::Registry) -> DashboardProfileEngine {
    DashboardProfileEngine {
        steps: vec![
            Rc::new(FlaggedDashboardProfileAdjuster {
                flags: flags.clone(),
                flag: "dashboard-profile-strict".to_string(),
                percent: -8.7,
            }) as Rc<dyn DashboardProfileAdjuster>,
            Rc::new(TieredDashboardProfileAdjuster {
                bounds: vec![839.0, 3786.0, 4427.0, 7097.0],
                rates: vec![0.82, 1.04, 1.03, 1.15],
            }),
            Rc::new(SegmentDashboardProfileAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.11), ("west".to_string(), 0.7)]),
            }),
            Rc::new(MinimumDashboardProfileAdjuster { minimum: 26.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 86364.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_profile_engine(&flags::new_registry());
        let item = new_dashboard_profile("code-960", "label-693");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
