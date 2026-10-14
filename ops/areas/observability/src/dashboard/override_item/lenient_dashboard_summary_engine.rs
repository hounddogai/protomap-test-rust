use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient dashboard summary calculation.
pub trait LenientDashboardSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientDashboardSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardSummaryMaximumOffsetAdjuster {
    pub weight: f64,
}

impl LenientDashboardSummaryAdjuster for LenientDashboardSummaryMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &LenientDashboardSummary, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientDashboardSummaryAdjuster {
    pub percent: f64,
}

impl LenientDashboardSummaryAdjuster for PercentLenientDashboardSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientDashboardSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientDashboardSummaryAdjuster {
    pub step: f64,
}

impl LenientDashboardSummaryAdjuster for RoundingLenientDashboardSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientDashboardSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientDashboardSummaryAdjuster {
    pub minimum: f64,
}

impl LenientDashboardSummaryAdjuster for MinimumLenientDashboardSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientDashboardSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientDashboardSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientDashboardSummaryAdjuster for FlaggedLenientDashboardSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientDashboardSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient dashboard summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientDashboardSummaryEngine {
    pub steps: Vec<Rc<dyn LenientDashboardSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientDashboardSummaryEngine {
    pub fn price(&mut self, item: &LenientDashboardSummary) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[LenientDashboardSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_dashboard_summary_engine(flags: &flags::Registry) -> LenientDashboardSummaryEngine {
    LenientDashboardSummaryEngine {
        steps: vec![
            Rc::new(LenientDashboardSummaryMaximumOffsetAdjuster { weight: 0.16 })
                as Rc<dyn LenientDashboardSummaryAdjuster>,
            Rc::new(PercentLenientDashboardSummaryAdjuster { percent: 23.5 }),
            Rc::new(RoundingLenientDashboardSummaryAdjuster { step: 0.25 }),
            Rc::new(MinimumLenientDashboardSummaryAdjuster { minimum: 5.0 }),
            Rc::new(FlaggedLenientDashboardSummaryAdjuster {
                flags: flags.clone(),
                flag: "lenient-dashboard-summary-regional".to_string(),
                percent: 4.9,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 30397.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_dashboard_summary_engine(&flags::new_registry());
        let item = new_lenient_dashboard_summary("code-148", "title-38");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
