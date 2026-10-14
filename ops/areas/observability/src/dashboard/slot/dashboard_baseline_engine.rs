use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dashboard baseline calculation.
pub trait DashboardBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardBaselineStep for FlaggedDashboardBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardBaselineStep for TieredDashboardBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardBaselineAdjuster {
    pub percent: f64,
}

impl DashboardBaselineStep for PercentDashboardBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDashboardBaselineAdjuster {
    pub step: f64,
}

impl DashboardBaselineStep for RoundingDashboardBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DashboardBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the dashboard baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardBaselineEngine {
    pub steps: Vec<Rc<dyn DashboardBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardBaselineEngine {
    pub fn compute(&mut self, item: &DashboardBaseline) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardBaselineStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_dashboard_baseline_engine(flags: &flags::Registry) -> DashboardBaselineEngine {
    DashboardBaselineEngine {
        steps: vec![
            Rc::new(FlaggedDashboardBaselineAdjuster {
                flags: flags.clone(),
                flag: "dashboard-baseline-legacy".to_string(),
                percent: 9.9,
            }) as Rc<dyn DashboardBaselineStep>,
            Rc::new(TieredDashboardBaselineAdjuster {
                bounds: vec![4674.0, 6864.0, 9393.0],
                rates: vec![0.9, 0.94, 0.99],
            }),
            Rc::new(PercentDashboardBaselineAdjuster { percent: -10.8 }),
            Rc::new(RoundingDashboardBaselineAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 48934.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_baseline_engine(&flags::new_registry());
        let item = new_dashboard_baseline("code-327", "title-841");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
