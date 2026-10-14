use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped dashboard tier calculation.
pub trait CappedDashboardTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedDashboardTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryCappedDashboardTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedDashboardTierAdjuster for CategoryCappedDashboardTierAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &CappedDashboardTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedDashboardTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedDashboardTierAdjuster for FlaggedCappedDashboardTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedDashboardTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedDashboardTierAdjuster {
    pub cap: f64,
}

impl CappedDashboardTierAdjuster for CapCappedDashboardTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedDashboardTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedDashboardTierAdjuster {
    pub percent: f64,
}

impl CappedDashboardTierAdjuster for PercentCappedDashboardTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedDashboardTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedDashboardTierAdjuster {
    pub step: f64,
}

impl CappedDashboardTierAdjuster for RoundingCappedDashboardTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedDashboardTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the capped dashboard tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedDashboardTierEngine {
    pub steps: Vec<Rc<dyn CappedDashboardTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedDashboardTierEngine {
    pub fn evaluate(&mut self, item: &CappedDashboardTier) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[CappedDashboardTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_dashboard_tier_engine(flags: &flags::Registry) -> CappedDashboardTierEngine {
    CappedDashboardTierEngine {
        steps: vec![
            Rc::new(CategoryCappedDashboardTierAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.16)]),
            }) as Rc<dyn CappedDashboardTierAdjuster>,
            Rc::new(FlaggedCappedDashboardTierAdjuster {
                flags: flags.clone(),
                flag: "capped-dashboard-tier-regional".to_string(),
                percent: 1.0,
            }),
            Rc::new(CapCappedDashboardTierAdjuster { cap: 45206.0 }),
            Rc::new(PercentCappedDashboardTierAdjuster { percent: -9.7 }),
            Rc::new(RoundingCappedDashboardTierAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 5124.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_dashboard_tier_engine(&flags::new_registry());
        let item = new_capped_dashboard_tier("code-940", "label-988");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
