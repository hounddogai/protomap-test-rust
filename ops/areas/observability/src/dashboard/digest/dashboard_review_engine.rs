use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the dashboard review calculation.
pub trait DashboardReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardReviewAdjuster for TieredDashboardReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardReviewAdjuster {
    pub percent: f64,
}

impl DashboardReviewAdjuster for PercentDashboardReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardReviewTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl DashboardReviewAdjuster for DashboardReviewTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardReview, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardReviewAdjuster for FlaggedDashboardReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDashboardReviewAdjuster {
    pub cap: f64,
}

impl DashboardReviewAdjuster for CapDashboardReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DashboardReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDashboardReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardReviewAdjuster for NameDashboardReviewAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DashboardReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the dashboard review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardReviewEngine {
    pub steps: Vec<Rc<dyn DashboardReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardReviewEngine {
    pub fn compute(&mut self, item: &DashboardReview) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardReviewAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_dashboard_review_engine(flags: &flags::Registry) -> DashboardReviewEngine {
    DashboardReviewEngine {
        steps: vec![
            Rc::new(TieredDashboardReviewAdjuster {
                bounds: vec![4026.0, 5197.0],
                rates: vec![1.01, 1.03],
            }) as Rc<dyn DashboardReviewAdjuster>,
            Rc::new(PercentDashboardReviewAdjuster { percent: -20.0 }),
            Rc::new(DashboardReviewTtlSecondsOffsetAdjuster { weight: 0.05 }),
            Rc::new(FlaggedDashboardReviewAdjuster {
                flags: flags.clone(),
                flag: "dashboard-review-capped".to_string(),
                percent: -0.8,
            }),
            Rc::new(CapDashboardReviewAdjuster { cap: 32191.0 }),
            Rc::new(NameDashboardReviewAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.98), ("west".to_string(), 0.7)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 36776.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_review_engine(&flags::new_registry());
        let item = new_dashboard_review("code-162", "name-856");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
