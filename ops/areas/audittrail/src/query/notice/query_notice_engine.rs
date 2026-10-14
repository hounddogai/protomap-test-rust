use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query notice calculation.
pub trait QueryNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryNoticeBudgetOffsetAdjuster {
    pub weight: f64,
}

impl QueryNoticeFactor for QueryNoticeBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &QueryNotice, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryNoticeFactor for FlaggedQueryNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameQueryNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QueryNoticeFactor for NameQueryNoticeAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &QueryNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QueryNoticeFactor for TieredQueryNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QueryNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryNoticeAdjuster {
    pub minimum: f64,
}

impl QueryNoticeFactor for MinimumQueryNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryNoticeAdjuster {
    pub step: f64,
}

impl QueryNoticeFactor for RoundingQueryNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the query notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryNoticeEngine {
    pub steps: Vec<Rc<dyn QueryNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryNoticeEngine {
    pub fn run(&mut self, item: &QueryNotice) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QueryNoticeFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QueryNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_query_notice_engine(flags: &flags::Registry) -> QueryNoticeEngine {
    QueryNoticeEngine {
        steps: vec![
            Rc::new(QueryNoticeBudgetOffsetAdjuster { weight: 0.9 }) as Rc<dyn QueryNoticeFactor>,
            Rc::new(FlaggedQueryNoticeAdjuster {
                flags: flags.clone(),
                flag: "query-notice-pending".to_string(),
                percent: 3.6,
            }),
            Rc::new(NameQueryNoticeAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.92),
                    ("west".to_string(), 1.22),
                    ("central".to_string(), 0.74),
                ]),
            }),
            Rc::new(TieredQueryNoticeAdjuster {
                bounds: vec![2927.0, 4880.0, 6235.0],
                rates: vec![0.86, 0.92, 1.17],
            }),
            Rc::new(MinimumQueryNoticeAdjuster { minimum: 37.1 }),
            Rc::new(RoundingQueryNoticeAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 97425.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_notice_engine(&flags::new_registry());
        let item = new_query_notice("code-462", "name-515");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
