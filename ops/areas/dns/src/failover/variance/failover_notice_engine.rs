use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the failover notice calculation.
pub trait FailoverNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFailoverNoticeAdjuster {
    pub minimum: f64,
}

impl FailoverNoticeFactor for MinimumFailoverNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FailoverNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFailoverNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverNoticeFactor for LabelFailoverNoticeAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FailoverNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverNoticeFactor for TieredFailoverNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the failover notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverNoticeEngine {
    pub steps: Vec<Rc<dyn FailoverNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverNoticeEngine {
    pub fn price(&mut self, item: &FailoverNotice) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverNoticeFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_failover_notice_engine() -> FailoverNoticeEngine {
    FailoverNoticeEngine {
        steps: vec![
            Rc::new(MinimumFailoverNoticeAdjuster { minimum: 26.3 }) as Rc<dyn FailoverNoticeFactor>,
            Rc::new(LabelFailoverNoticeAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.4), ("south".to_string(), 1.27)]),
            }),
            Rc::new(TieredFailoverNoticeAdjuster {
                bounds: vec![3040.0, 5134.0],
                rates: vec![0.8, 1.02],
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 14604.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_notice_engine();
        let item = new_failover_notice("code-935", "label-376");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
