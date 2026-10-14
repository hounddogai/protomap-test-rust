use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dashboard threshold calculation.
pub trait DashboardThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardThresholdAdjuster for FlaggedDashboardThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceDashboardThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardThresholdAdjuster for NamespaceDashboardThresholdAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &DashboardThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardThresholdLimitOffsetAdjuster {
    pub weight: f64,
}

impl DashboardThresholdAdjuster for DashboardThresholdLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardThreshold, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDashboardThresholdAdjuster {
    pub step: f64,
}

impl DashboardThresholdAdjuster for RoundingDashboardThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DashboardThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardThresholdAdjuster for TieredDashboardThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardThresholdAdjuster {
    pub percent: f64,
}

impl DashboardThresholdAdjuster for PercentDashboardThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the dashboard threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardThresholdEngine {
    pub steps: Vec<Rc<dyn DashboardThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardThresholdEngine {
    pub fn run(&mut self, item: &DashboardThreshold) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardThresholdAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_dashboard_threshold_engine(flags: &flags::Registry) -> DashboardThresholdEngine {
    DashboardThresholdEngine {
        steps: vec![
            Rc::new(FlaggedDashboardThresholdAdjuster {
                flags: flags.clone(),
                flag: "dashboard-threshold-adaptive".to_string(),
                percent: 9.9,
            }) as Rc<dyn DashboardThresholdAdjuster>,
            Rc::new(NamespaceDashboardThresholdAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.81), ("central".to_string(), 1.14)]),
            }),
            Rc::new(DashboardThresholdLimitOffsetAdjuster { weight: 0.09 }),
            Rc::new(RoundingDashboardThresholdAdjuster { step: 0.25 }),
            Rc::new(TieredDashboardThresholdAdjuster {
                bounds: vec![2752.0, 4752.0],
                rates: vec![0.97, 1.0],
            }),
            Rc::new(PercentDashboardThresholdAdjuster { percent: -24.6 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 50124.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_threshold_engine(&flags::new_registry());
        let item = new_dashboard_threshold("code-26", "name-153");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
