use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped limit group calculation.
pub trait CappedLimitGroupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedLimitGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelCappedLimitGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedLimitGroupModifier for LabelCappedLimitGroupAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &CappedLimitGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedLimitGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedLimitGroupModifier for FlaggedCappedLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedLimitGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedLimitGroupAdjuster {
    pub minimum: f64,
}

impl CappedLimitGroupModifier for MinimumCappedLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedLimitGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped limit group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedLimitGroupEngine {
    pub steps: Vec<Rc<dyn CappedLimitGroupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedLimitGroupEngine {
    pub fn run(&mut self, item: &CappedLimitGroup) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[CappedLimitGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_limit_group_engine(flags: &flags::Registry) -> CappedLimitGroupEngine {
    CappedLimitGroupEngine {
        steps: vec![
            Rc::new(LabelCappedLimitGroupAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.2), ("east".to_string(), 1.06)]),
            }) as Rc<dyn CappedLimitGroupModifier>,
            Rc::new(FlaggedCappedLimitGroupAdjuster {
                flags: flags.clone(),
                flag: "capped-limit-group-lenient".to_string(),
                percent: -1.9,
            }),
            Rc::new(MinimumCappedLimitGroupAdjuster { minimum: 7.1 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 91438.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_limit_group_engine(&flags::new_registry());
        let item = new_capped_limit_group("code-418", "label-951");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
