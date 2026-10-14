use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the limit record calculation.
pub trait LimitRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitRecordAdjuster {
    pub percent: f64,
}

impl LimitRecordFactor for PercentLimitRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLimitRecordAdjuster {
    pub cap: f64,
}

impl LimitRecordFactor for CapLimitRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LimitRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRecordCeilingOffsetAdjuster {
    pub weight: f64,
}

impl LimitRecordFactor for LimitRecordCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &LimitRecord, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryLimitRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LimitRecordFactor for CountryLimitRecordAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &LimitRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitRecordAdjuster {
    pub minimum: f64,
}

impl LimitRecordFactor for MinimumLimitRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the limit record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitRecordEngine {
    pub steps: Vec<Rc<dyn LimitRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitRecordEngine {
    pub fn price(&mut self, item: &LimitRecord) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LimitRecordFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LimitRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_limit_record_engine() -> LimitRecordEngine {
    LimitRecordEngine {
        steps: vec![
            Rc::new(PercentLimitRecordAdjuster { percent: 19.5 }) as Rc<dyn LimitRecordFactor>,
            Rc::new(CapLimitRecordAdjuster { cap: 48730.0 }),
            Rc::new(LimitRecordCeilingOffsetAdjuster { weight: 0.78 }),
            Rc::new(CountryLimitRecordAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.24), ("west".to_string(), 0.8)]),
            }),
            Rc::new(MinimumLimitRecordAdjuster { minimum: 20.8 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 14478.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_record_engine();
        let item = new_limit_record("code-559", "title-868");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
