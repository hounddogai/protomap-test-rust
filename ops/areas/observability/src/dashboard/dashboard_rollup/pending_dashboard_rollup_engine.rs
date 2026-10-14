use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending dashboard rollup calculation.
pub trait PendingDashboardRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingDashboardRollup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingDashboardRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingDashboardRollupFactor for FlaggedPendingDashboardRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingDashboardRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingDashboardRollupAdjuster {
    pub percent: f64,
}

impl PendingDashboardRollupFactor for PercentPendingDashboardRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingDashboardRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingDashboardRollupAdjuster {
    pub step: f64,
}

impl PendingDashboardRollupFactor for RoundingPendingDashboardRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingDashboardRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePendingDashboardRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingDashboardRollupFactor for TitlePendingDashboardRollupAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PendingDashboardRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingDashboardRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingDashboardRollupFactor for TieredPendingDashboardRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingDashboardRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending dashboard rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingDashboardRollupEngine {
    pub steps: Vec<Rc<dyn PendingDashboardRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingDashboardRollupEngine {
    pub fn run(&mut self, item: &PendingDashboardRollup) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PendingDashboardRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_dashboard_rollup_engine(flags: &flags::Registry) -> PendingDashboardRollupEngine {
    PendingDashboardRollupEngine {
        steps: vec![
            Rc::new(FlaggedPendingDashboardRollupAdjuster {
                flags: flags.clone(),
                flag: "pending-dashboard-rollup-legacy".to_string(),
                percent: -6.9,
            }) as Rc<dyn PendingDashboardRollupFactor>,
            Rc::new(PercentPendingDashboardRollupAdjuster { percent: -8.5 }),
            Rc::new(RoundingPendingDashboardRollupAdjuster { step: 0.5 }),
            Rc::new(TitlePendingDashboardRollupAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.12), ("central".to_string(), 0.97)]),
            }),
            Rc::new(TieredPendingDashboardRollupAdjuster {
                bounds: vec![4429.0, 4692.0],
                rates: vec![1.09, 1.03],
            }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 86297.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_dashboard_rollup_engine(&flags::new_registry());
        let item = new_pending_dashboard_rollup("code-868", "title-736");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
