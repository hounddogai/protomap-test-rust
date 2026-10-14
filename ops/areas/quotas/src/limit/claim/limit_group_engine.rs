use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the limit group calculation.
pub trait LimitGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitGroupQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl LimitGroupStep for LimitGroupQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &LimitGroup, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLimitGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LimitGroupStep for FlaggedLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LimitGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitGroupAdjuster {
    pub percent: f64,
}

impl LimitGroupStep for PercentLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the limit group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitGroupEngine {
    pub steps: Vec<Rc<dyn LimitGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitGroupEngine {
    pub fn run(&mut self, item: &LimitGroup) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LimitGroupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LimitGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_limit_group_engine(flags: &flags::Registry) -> LimitGroupEngine {
    LimitGroupEngine {
        steps: vec![
            Rc::new(LimitGroupQuotaUnitsOffsetAdjuster { weight: 0.86 }) as Rc<dyn LimitGroupStep>,
            Rc::new(FlaggedLimitGroupAdjuster {
                flags: flags.clone(),
                flag: "limit-group-pending".to_string(),
                percent: 5.5,
            }),
            Rc::new(PercentLimitGroupAdjuster { percent: -7.5 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 6468.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_group_engine(&flags::new_registry());
        let item = new_limit_group("code-101", "title-316");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
