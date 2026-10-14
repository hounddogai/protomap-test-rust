use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default dashboard digest calculation.
pub trait DefaultDashboardDigestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultDashboardDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardDigestGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl DefaultDashboardDigestStep for DefaultDashboardDigestGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultDashboardDigest, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultDashboardDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultDashboardDigestStep for FlaggedDefaultDashboardDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultDashboardDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultDashboardDigestAdjuster {
    pub cap: f64,
}

impl DefaultDashboardDigestStep for CapDefaultDashboardDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultDashboardDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultDashboardDigestAdjuster {
    pub minimum: f64,
}

impl DefaultDashboardDigestStep for MinimumDefaultDashboardDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultDashboardDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDefaultDashboardDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultDashboardDigestStep for TitleDefaultDashboardDigestAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DefaultDashboardDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default dashboard digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultDashboardDigestEngine {
    pub steps: Vec<Rc<dyn DefaultDashboardDigestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultDashboardDigestEngine {
    pub fn run(&mut self, item: &DefaultDashboardDigest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DefaultDashboardDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_dashboard_digest_engine(flags: &flags::Registry) -> DefaultDashboardDigestEngine {
    DefaultDashboardDigestEngine {
        steps: vec![
            Rc::new(DefaultDashboardDigestGraceDaysOffsetAdjuster { weight: 0.09 })
                as Rc<dyn DefaultDashboardDigestStep>,
            Rc::new(FlaggedDefaultDashboardDigestAdjuster {
                flags: flags.clone(),
                flag: "default-dashboard-digest-lenient".to_string(),
                percent: 6.2,
            }),
            Rc::new(CapDefaultDashboardDigestAdjuster { cap: 44531.0 }),
            Rc::new(MinimumDefaultDashboardDigestAdjuster { minimum: 44.9 }),
            Rc::new(TitleDefaultDashboardDigestAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.26)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 72547.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_dashboard_digest_engine(&flags::new_registry());
        let item = new_default_dashboard_digest("code-274", "title-858");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
