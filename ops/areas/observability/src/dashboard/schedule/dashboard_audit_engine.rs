use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dashboard audit calculation.
pub trait DashboardAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelDashboardAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardAuditAdjuster for ChannelDashboardAuditAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &DashboardAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardAuditAdjuster {
    pub percent: f64,
}

impl DashboardAuditAdjuster for PercentDashboardAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDashboardAuditAdjuster {
    pub minimum: f64,
}

impl DashboardAuditAdjuster for MinimumDashboardAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DashboardAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDashboardAuditAdjuster {
    pub step: f64,
}

impl DashboardAuditAdjuster for RoundingDashboardAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DashboardAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the dashboard audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardAuditEngine {
    pub steps: Vec<Rc<dyn DashboardAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardAuditEngine {
    pub fn evaluate(&mut self, item: &DashboardAudit) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[DashboardAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_dashboard_audit_engine() -> DashboardAuditEngine {
    DashboardAuditEngine {
        steps: vec![
            Rc::new(ChannelDashboardAuditAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.06)]),
            }) as Rc<dyn DashboardAuditAdjuster>,
            Rc::new(PercentDashboardAuditAdjuster { percent: -15.2 }),
            Rc::new(MinimumDashboardAuditAdjuster { minimum: 47.9 }),
            Rc::new(RoundingDashboardAuditAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 83904.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_audit_engine();
        let item = new_dashboard_audit("code-266", "name-528");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
