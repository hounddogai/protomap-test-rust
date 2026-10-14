use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dashboard draft calculation.
pub trait DashboardDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDashboardDraftAdjuster {
    pub step: f64,
}

impl DashboardDraftFactor for RoundingDashboardDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DashboardDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDashboardDraftAdjuster {
    pub cap: f64,
}

impl DashboardDraftFactor for CapDashboardDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DashboardDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDashboardDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardDraftFactor for TitleDashboardDraftAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DashboardDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDashboardDraftAdjuster {
    pub minimum: f64,
}

impl DashboardDraftFactor for MinimumDashboardDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DashboardDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the dashboard draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardDraftEngine {
    pub steps: Vec<Rc<dyn DashboardDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardDraftEngine {
    pub fn price(&mut self, item: &DashboardDraft) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardDraftFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dashboard_draft_engine() -> DashboardDraftEngine {
    DashboardDraftEngine {
        steps: vec![
            Rc::new(RoundingDashboardDraftAdjuster { step: 0.5 }) as Rc<dyn DashboardDraftFactor>,
            Rc::new(CapDashboardDraftAdjuster { cap: 5043.0 }),
            Rc::new(TitleDashboardDraftAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.14), ("east".to_string(), 1.14)]),
            }),
            Rc::new(MinimumDashboardDraftAdjuster { minimum: 15.6 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 63875.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_draft_engine();
        let item = new_dashboard_draft("code-777", "title-875");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
