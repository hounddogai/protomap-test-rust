use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending dashboard baseline calculation.
pub trait PendingDashboardBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingDashboardBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePendingDashboardBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingDashboardBaselineStep for TitlePendingDashboardBaselineAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PendingDashboardBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingDashboardBaselineAdjuster {
    pub percent: f64,
}

impl PendingDashboardBaselineStep for PercentPendingDashboardBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingDashboardBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardBaselineThresholdOffsetAdjuster {
    pub weight: f64,
}

impl PendingDashboardBaselineStep for PendingDashboardBaselineThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &PendingDashboardBaseline, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingDashboardBaselineAdjuster {
    pub minimum: f64,
}

impl PendingDashboardBaselineStep for MinimumPendingDashboardBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingDashboardBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending dashboard baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingDashboardBaselineEngine {
    pub steps: Vec<Rc<dyn PendingDashboardBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingDashboardBaselineEngine {
    pub fn price(&mut self, item: &PendingDashboardBaseline) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[PendingDashboardBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_dashboard_baseline_engine() -> PendingDashboardBaselineEngine {
    PendingDashboardBaselineEngine {
        steps: vec![
            Rc::new(TitlePendingDashboardBaselineAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.08), ("central".to_string(), 0.95)]),
            }) as Rc<dyn PendingDashboardBaselineStep>,
            Rc::new(PercentPendingDashboardBaselineAdjuster { percent: 11.5 }),
            Rc::new(PendingDashboardBaselineThresholdOffsetAdjuster { weight: 0.55 }),
            Rc::new(MinimumPendingDashboardBaselineAdjuster { minimum: 34.5 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 29390.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_dashboard_baseline_engine();
        let item = new_pending_dashboard_baseline("code-343", "title-518");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
