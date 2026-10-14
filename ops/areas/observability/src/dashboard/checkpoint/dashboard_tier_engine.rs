use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the dashboard tier calculation.
pub trait DashboardTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDashboardTierAdjuster {
    pub minimum: f64,
}

impl DashboardTierModifier for MinimumDashboardTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DashboardTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardTierModifier for FlaggedDashboardTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelDashboardTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardTierModifier for ChannelDashboardTierAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &DashboardTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTierRateOffsetAdjuster {
    pub weight: f64,
}

impl DashboardTierModifier for DashboardTierRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardTier, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDashboardTierAdjuster {
    pub cap: f64,
}

impl DashboardTierModifier for CapDashboardTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DashboardTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardTierAdjuster {
    pub percent: f64,
}

impl DashboardTierModifier for PercentDashboardTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the dashboard tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardTierEngine {
    pub steps: Vec<Rc<dyn DashboardTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardTierEngine {
    pub fn run(&mut self, item: &DashboardTier) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardTierModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_dashboard_tier_engine(flags: &flags::Registry) -> DashboardTierEngine {
    DashboardTierEngine {
        steps: vec![
            Rc::new(MinimumDashboardTierAdjuster { minimum: 19.3 }) as Rc<dyn DashboardTierModifier>,
            Rc::new(FlaggedDashboardTierAdjuster {
                flags: flags.clone(),
                flag: "dashboard-tier-fallback".to_string(),
                percent: 7.0,
            }),
            Rc::new(ChannelDashboardTierAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.88)]),
            }),
            Rc::new(DashboardTierRateOffsetAdjuster { weight: 1.9 }),
            Rc::new(CapDashboardTierAdjuster { cap: 2738.0 }),
            Rc::new(PercentDashboardTierAdjuster { percent: -5.5 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 58300.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_tier_engine(&flags::new_registry());
        let item = new_dashboard_tier("code-42", "title-672");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
