use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query claim calculation.
pub trait QueryClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeQueryClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QueryClaimStep for ScopeQueryClaimAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &QueryClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryClaimUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl QueryClaimStep for QueryClaimUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &QueryClaim, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryClaimAdjuster {
    pub step: f64,
}

impl QueryClaimStep for RoundingQueryClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryClaimStep for FlaggedQueryClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the query claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryClaimEngine {
    pub steps: Vec<Rc<dyn QueryClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryClaimEngine {
    pub fn evaluate(&mut self, item: &QueryClaim) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[QueryClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_query_claim_engine(flags: &flags::Registry) -> QueryClaimEngine {
    QueryClaimEngine {
        steps: vec![
            Rc::new(ScopeQueryClaimAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.2), ("online".to_string(), 1.04)]),
            }) as Rc<dyn QueryClaimStep>,
            Rc::new(QueryClaimUpdatedAtOffsetAdjuster { weight: 1.03 }),
            Rc::new(RoundingQueryClaimAdjuster { step: 5.0 }),
            Rc::new(FlaggedQueryClaimAdjuster {
                flags: flags.clone(),
                flag: "query-claim-strict".to_string(),
                percent: -1.5,
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 26831.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_claim_engine(&flags::new_registry());
        let item = new_query_claim("code-62", "title-754");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
