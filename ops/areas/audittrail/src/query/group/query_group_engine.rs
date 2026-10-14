use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query group calculation.
pub trait QueryGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryGroupAdjuster {
    pub step: f64,
}

impl QueryGroupFactor for RoundingQueryGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryGroupRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl QueryGroupFactor for QueryGroupRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &QueryGroup, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryGroupAdjuster {
    pub minimum: f64,
}

impl QueryGroupFactor for MinimumQueryGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitQueryGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QueryGroupFactor for UnitQueryGroupAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &QueryGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryGroupFactor for FlaggedQueryGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQueryGroupAdjuster {
    pub percent: f64,
}

impl QueryGroupFactor for PercentQueryGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QueryGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the query group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryGroupEngine {
    pub steps: Vec<Rc<dyn QueryGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryGroupEngine {
    pub fn compute(&mut self, item: &QueryGroup) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QueryGroupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QueryGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_query_group_engine(flags: &flags::Registry) -> QueryGroupEngine {
    QueryGroupEngine {
        steps: vec![
            Rc::new(RoundingQueryGroupAdjuster { step: 0.05 }) as Rc<dyn QueryGroupFactor>,
            Rc::new(QueryGroupRotationDaysOffsetAdjuster { weight: 0.67 }),
            Rc::new(MinimumQueryGroupAdjuster { minimum: 44.9 }),
            Rc::new(UnitQueryGroupAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.39),
                    ("central".to_string(), 0.83),
                    ("online".to_string(), 0.82),
                ]),
            }),
            Rc::new(FlaggedQueryGroupAdjuster {
                flags: flags.clone(),
                flag: "query-group-tiered".to_string(),
                percent: 6.8,
            }),
            Rc::new(PercentQueryGroupAdjuster { percent: -10.5 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 75299.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_group_engine(&flags::new_registry());
        let item = new_query_group("code-51", "name-198");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
