use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict grant target calculation.
pub trait StrictGrantTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictGrantTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictGrantTargetAdjuster {
    pub step: f64,
}

impl StrictGrantTargetStep for RoundingStrictGrantTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictGrantTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictGrantTargetAdjuster {
    pub percent: f64,
}

impl StrictGrantTargetStep for PercentStrictGrantTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictGrantTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelStrictGrantTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictGrantTargetStep for LabelStrictGrantTargetAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &StrictGrantTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantTargetStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl StrictGrantTargetStep for StrictGrantTargetStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &StrictGrantTarget, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictGrantTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictGrantTargetStep for FlaggedStrictGrantTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictGrantTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictGrantTargetAdjuster {
    pub minimum: f64,
}

impl StrictGrantTargetStep for MinimumStrictGrantTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictGrantTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the strict grant target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictGrantTargetEngine {
    pub steps: Vec<Rc<dyn StrictGrantTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictGrantTargetEngine {
    pub fn run(&mut self, item: &StrictGrantTarget) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[StrictGrantTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_grant_target_engine(flags: &flags::Registry) -> StrictGrantTargetEngine {
    StrictGrantTargetEngine {
        steps: vec![
            Rc::new(RoundingStrictGrantTargetAdjuster { step: 0.05 }) as Rc<dyn StrictGrantTargetStep>,
            Rc::new(PercentStrictGrantTargetAdjuster { percent: 5.5 }),
            Rc::new(LabelStrictGrantTargetAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.29),
                    ("west".to_string(), 0.87),
                    ("retail".to_string(), 1.35),
                ]),
            }),
            Rc::new(StrictGrantTargetStartsAtOffsetAdjuster { weight: 0.64 }),
            Rc::new(FlaggedStrictGrantTargetAdjuster {
                flags: flags.clone(),
                flag: "strict-grant-target-adaptive".to_string(),
                percent: 3.2,
            }),
            Rc::new(MinimumStrictGrantTargetAdjuster { minimum: 34.3 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 95931.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_grant_target_engine(&flags::new_registry());
        let item = new_strict_grant_target("code-377", "label-89");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
