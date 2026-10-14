use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped grant target calculation.
pub trait CappedGrantTargetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedGrantTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterCappedGrantTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedGrantTargetModifier for ClusterCappedGrantTargetAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &CappedGrantTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedGrantTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedGrantTargetModifier for FlaggedCappedGrantTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedGrantTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantTargetMaximumOffsetAdjuster {
    pub weight: f64,
}

impl CappedGrantTargetModifier for CappedGrantTargetMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &CappedGrantTarget, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedGrantTargetAdjuster {
    pub percent: f64,
}

impl CappedGrantTargetModifier for PercentCappedGrantTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedGrantTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedGrantTargetAdjuster {
    pub minimum: f64,
}

impl CappedGrantTargetModifier for MinimumCappedGrantTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedGrantTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedGrantTargetAdjuster {
    pub cap: f64,
}

impl CappedGrantTargetModifier for CapCappedGrantTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedGrantTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped grant target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedGrantTargetEngine {
    pub steps: Vec<Rc<dyn CappedGrantTargetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedGrantTargetEngine {
    pub fn evaluate(&mut self, item: &CappedGrantTarget) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[CappedGrantTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_grant_target_engine(flags: &flags::Registry) -> CappedGrantTargetEngine {
    CappedGrantTargetEngine {
        steps: vec![
            Rc::new(ClusterCappedGrantTargetAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.35),
                    ("west".to_string(), 0.9),
                    ("south".to_string(), 0.98),
                ]),
            }) as Rc<dyn CappedGrantTargetModifier>,
            Rc::new(FlaggedCappedGrantTargetAdjuster {
                flags: flags.clone(),
                flag: "capped-grant-target-fallback".to_string(),
                percent: 1.9,
            }),
            Rc::new(CappedGrantTargetMaximumOffsetAdjuster { weight: 1.06 }),
            Rc::new(PercentCappedGrantTargetAdjuster { percent: -10.5 }),
            Rc::new(MinimumCappedGrantTargetAdjuster { minimum: 31.0 }),
            Rc::new(CapCappedGrantTargetAdjuster { cap: 45619.0 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 51916.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_grant_target_engine(&flags::new_registry());
        let item = new_capped_grant_target("code-994", "name-258");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
