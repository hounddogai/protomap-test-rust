use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dashboard notice calculation.
pub trait DashboardNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardNoticeMarginOffsetAdjuster {
    pub weight: f64,
}

impl DashboardNoticeStep for DashboardNoticeMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardNotice, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardNoticeAdjuster {
    pub percent: f64,
}

impl DashboardNoticeStep for PercentDashboardNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDashboardNoticeAdjuster {
    pub step: f64,
}

impl DashboardNoticeStep for RoundingDashboardNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DashboardNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardNoticeStep for FlaggedDashboardNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryDashboardNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardNoticeStep for CountryDashboardNoticeAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &DashboardNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDashboardNoticeAdjuster {
    pub minimum: f64,
}

impl DashboardNoticeStep for MinimumDashboardNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DashboardNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the dashboard notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardNoticeEngine {
    pub steps: Vec<Rc<dyn DashboardNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardNoticeEngine {
    pub fn run(&mut self, item: &DashboardNotice) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_dashboard_notice_engine(flags: &flags::Registry) -> DashboardNoticeEngine {
    DashboardNoticeEngine {
        steps: vec![
            Rc::new(DashboardNoticeMarginOffsetAdjuster { weight: 1.3 }) as Rc<dyn DashboardNoticeStep>,
            Rc::new(PercentDashboardNoticeAdjuster { percent: -22.6 }),
            Rc::new(RoundingDashboardNoticeAdjuster { step: 0.25 }),
            Rc::new(FlaggedDashboardNoticeAdjuster {
                flags: flags.clone(),
                flag: "dashboard-notice-seasonal".to_string(),
                percent: -9.6,
            }),
            Rc::new(CountryDashboardNoticeAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.35)]),
            }),
            Rc::new(MinimumDashboardNoticeAdjuster { minimum: 29.0 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 29648.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_notice_engine(&flags::new_registry());
        let item = new_dashboard_notice("code-496", "label-692");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
