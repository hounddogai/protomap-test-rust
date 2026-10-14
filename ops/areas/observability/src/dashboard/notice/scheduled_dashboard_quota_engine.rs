use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled dashboard quota calculation.
pub trait ScheduledDashboardQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledDashboardQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledDashboardQuotaAdjuster {
    pub cap: f64,
}

impl ScheduledDashboardQuotaFactor for CapScheduledDashboardQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledDashboardQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledDashboardQuotaAdjuster {
    pub step: f64,
}

impl ScheduledDashboardQuotaFactor for RoundingScheduledDashboardQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledDashboardQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledDashboardQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledDashboardQuotaFactor for FlaggedScheduledDashboardQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledDashboardQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledDashboardQuotaAdjuster {
    pub minimum: f64,
}

impl ScheduledDashboardQuotaFactor for MinimumScheduledDashboardQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledDashboardQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledDashboardQuotaAdjuster {
    pub percent: f64,
}

impl ScheduledDashboardQuotaFactor for PercentScheduledDashboardQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledDashboardQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardQuotaEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledDashboardQuotaFactor for ScheduledDashboardQuotaEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledDashboardQuota, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

/// Runs the scheduled dashboard quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledDashboardQuotaEngine {
    pub steps: Vec<Rc<dyn ScheduledDashboardQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledDashboardQuotaEngine {
    pub fn evaluate(&mut self, item: &ScheduledDashboardQuota) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledDashboardQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_dashboard_quota_engine(flags: &flags::Registry) -> ScheduledDashboardQuotaEngine {
    ScheduledDashboardQuotaEngine {
        steps: vec![
            Rc::new(CapScheduledDashboardQuotaAdjuster { cap: 19592.0 }) as Rc<dyn ScheduledDashboardQuotaFactor>,
            Rc::new(RoundingScheduledDashboardQuotaAdjuster { step: 0.05 }),
            Rc::new(FlaggedScheduledDashboardQuotaAdjuster {
                flags: flags.clone(),
                flag: "scheduled-dashboard-quota-legacy".to_string(),
                percent: 6.6,
            }),
            Rc::new(MinimumScheduledDashboardQuotaAdjuster { minimum: 16.4 }),
            Rc::new(PercentScheduledDashboardQuotaAdjuster { percent: -11.2 }),
            Rc::new(ScheduledDashboardQuotaEndsAtOffsetAdjuster { weight: 0.41 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 42899.2,
    }
}
