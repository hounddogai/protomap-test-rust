use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rolling runbook summary calculation.
pub trait RollingRunbookSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingRunbookSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookSummaryDiscountOffsetAdjuster {
    pub weight: f64,
}

impl RollingRunbookSummaryAdjuster for RollingRunbookSummaryDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &RollingRunbookSummary, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRollingRunbookSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingRunbookSummaryAdjuster for TitleRollingRunbookSummaryAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RollingRunbookSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingRunbookSummaryAdjuster {
    pub minimum: f64,
}

impl RollingRunbookSummaryAdjuster for MinimumRollingRunbookSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingRunbookSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingRunbookSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingRunbookSummaryAdjuster for TieredRollingRunbookSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingRunbookSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling runbook summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingRunbookSummaryEngine {
    pub steps: Vec<Rc<dyn RollingRunbookSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingRunbookSummaryEngine {
    pub fn run(&mut self, item: &RollingRunbookSummary) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingRunbookSummaryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingRunbookSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_runbook_summary_engine() -> RollingRunbookSummaryEngine {
    RollingRunbookSummaryEngine {
        steps: vec![
            Rc::new(RollingRunbookSummaryDiscountOffsetAdjuster { weight: 0.15 })
                as Rc<dyn RollingRunbookSummaryAdjuster>,
            Rc::new(TitleRollingRunbookSummaryAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.72),
                    ("online".to_string(), 0.91),
                    ("east".to_string(), 1.1),
                ]),
            }),
            Rc::new(MinimumRollingRunbookSummaryAdjuster { minimum: 10.4 }),
            Rc::new(TieredRollingRunbookSummaryAdjuster {
                bounds: vec![4158.0, 4779.0],
                rates: vec![0.88, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 2124.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_runbook_summary_engine();
        let item = new_rolling_runbook_summary("code-929", "title-361");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
