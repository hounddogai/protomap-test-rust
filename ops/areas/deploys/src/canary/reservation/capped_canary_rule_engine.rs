use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped canary rule calculation.
pub trait CappedCanaryRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedCanaryRule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedCanaryRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedCanaryRuleModifier for FlaggedCappedCanaryRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedCanaryRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryRuleRatioOffsetAdjuster {
    pub weight: f64,
}

impl CappedCanaryRuleModifier for CappedCanaryRuleRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &CappedCanaryRule, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedCanaryRuleAdjuster {
    pub minimum: f64,
}

impl CappedCanaryRuleModifier for MinimumCappedCanaryRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedCanaryRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedCanaryRuleAdjuster {
    pub cap: f64,
}

impl CappedCanaryRuleModifier for CapCappedCanaryRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedCanaryRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedCanaryRuleAdjuster {
    pub percent: f64,
}

impl CappedCanaryRuleModifier for PercentCappedCanaryRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedCanaryRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped canary rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedCanaryRuleEngine {
    pub steps: Vec<Rc<dyn CappedCanaryRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedCanaryRuleEngine {
    pub fn run(&mut self, item: &CappedCanaryRule) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[CappedCanaryRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_canary_rule_engine(flags: &flags::Registry) -> CappedCanaryRuleEngine {
    CappedCanaryRuleEngine {
        steps: vec![
            Rc::new(FlaggedCappedCanaryRuleAdjuster {
                flags: flags.clone(),
                flag: "capped-canary-rule-regional".to_string(),
                percent: 8.0,
            }) as Rc<dyn CappedCanaryRuleModifier>,
            Rc::new(CappedCanaryRuleRatioOffsetAdjuster { weight: 1.92 }),
            Rc::new(MinimumCappedCanaryRuleAdjuster { minimum: 30.3 }),
            Rc::new(CapCappedCanaryRuleAdjuster { cap: 37425.0 }),
            Rc::new(PercentCappedCanaryRuleAdjuster { percent: -12.2 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 72998.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_canary_rule_engine(&flags::new_registry());
        let item = new_capped_canary_rule("code-460", "title-339");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
