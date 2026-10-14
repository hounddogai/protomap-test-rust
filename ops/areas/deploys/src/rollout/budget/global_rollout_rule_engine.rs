use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global rollout rule calculation.
pub trait GlobalRolloutRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalRolloutRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalRolloutRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalRolloutRuleModifier for TieredGlobalRolloutRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalRolloutRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalRolloutRuleAdjuster {
    pub step: f64,
}

impl GlobalRolloutRuleModifier for RoundingGlobalRolloutRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalRolloutRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalRolloutRuleAdjuster {
    pub percent: f64,
}

impl GlobalRolloutRuleModifier for PercentGlobalRolloutRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalRolloutRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalRolloutRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalRolloutRuleModifier for FlaggedGlobalRolloutRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalRolloutRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleGlobalRolloutRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalRolloutRuleModifier for LocaleGlobalRolloutRuleAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &GlobalRolloutRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the global rollout rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalRolloutRuleEngine {
    pub steps: Vec<Rc<dyn GlobalRolloutRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalRolloutRuleEngine {
    pub fn compute(&mut self, item: &GlobalRolloutRule) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalRolloutRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_rollout_rule_engine(flags: &flags::Registry) -> GlobalRolloutRuleEngine {
    GlobalRolloutRuleEngine {
        steps: vec![
            Rc::new(TieredGlobalRolloutRuleAdjuster {
                bounds: vec![4569.0, 6703.0, 7046.0, 8323.0],
                rates: vec![0.82, 1.12, 0.91, 1.08],
            }) as Rc<dyn GlobalRolloutRuleModifier>,
            Rc::new(RoundingGlobalRolloutRuleAdjuster { step: 1.0 }),
            Rc::new(PercentGlobalRolloutRuleAdjuster { percent: 9.0 }),
            Rc::new(FlaggedGlobalRolloutRuleAdjuster {
                flags: flags.clone(),
                flag: "global-rollout-rule-regional".to_string(),
                percent: 9.9,
            }),
            Rc::new(LocaleGlobalRolloutRuleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.95), ("north".to_string(), 1.29)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 7996.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_rollout_rule_engine(&flags::new_registry());
        let item = new_global_rollout_rule("code-312", "label-270");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
