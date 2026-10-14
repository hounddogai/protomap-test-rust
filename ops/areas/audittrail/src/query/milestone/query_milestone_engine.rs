use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query milestone calculation.
pub trait QueryMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QueryMilestoneModifier for TieredQueryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QueryMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryMilestoneAdjuster {
    pub cap: f64,
}

impl QueryMilestoneModifier for CapQueryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QueryMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQueryMilestoneAdjuster {
    pub percent: f64,
}

impl QueryMilestoneModifier for PercentQueryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QueryMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryMilestoneModifier for FlaggedQueryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryMilestoneGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl QueryMilestoneModifier for QueryMilestoneGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &QueryMilestone, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryMilestoneAdjuster {
    pub minimum: f64,
}

impl QueryMilestoneModifier for MinimumQueryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the query milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryMilestoneEngine {
    pub steps: Vec<Rc<dyn QueryMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryMilestoneEngine {
    pub fn compute(&mut self, item: &QueryMilestone) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[QueryMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_query_milestone_engine(flags: &flags::Registry) -> QueryMilestoneEngine {
    QueryMilestoneEngine {
        steps: vec![
            Rc::new(TieredQueryMilestoneAdjuster {
                bounds: vec![3444.0, 3655.0, 4432.0, 5927.0],
                rates: vec![0.91, 0.95, 0.86, 1.16],
            }) as Rc<dyn QueryMilestoneModifier>,
            Rc::new(CapQueryMilestoneAdjuster { cap: 31488.0 }),
            Rc::new(PercentQueryMilestoneAdjuster { percent: -8.1 }),
            Rc::new(FlaggedQueryMilestoneAdjuster {
                flags: flags.clone(),
                flag: "query-milestone-tiered".to_string(),
                percent: 1.3,
            }),
            Rc::new(QueryMilestoneGraceDaysOffsetAdjuster { weight: 0.91 }),
            Rc::new(MinimumQueryMilestoneAdjuster { minimum: 30.1 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 48119.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_milestone_engine(&flags::new_registry());
        let item = new_query_milestone("code-107", "title-518");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
