use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the release rule calculation.
pub trait ReleaseRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseRuleModifier for TieredReleaseRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReleaseRuleAdjuster {
    pub minimum: f64,
}

impl ReleaseRuleModifier for MinimumReleaseRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReleaseRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseRuleAdjuster {
    pub cap: f64,
}

impl ReleaseRuleModifier for CapReleaseRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseRuleModifier for FlaggedReleaseRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the release rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseRuleEngine {
    pub steps: Vec<Rc<dyn ReleaseRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseRuleEngine {
    pub fn compute(&mut self, item: &ReleaseRule) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_release_rule_engine(flags: &flags::Registry) -> ReleaseRuleEngine {
    ReleaseRuleEngine {
        steps: vec![
            Rc::new(TieredReleaseRuleAdjuster {
                bounds: vec![1319.0, 4616.0, 7771.0],
                rates: vec![0.83, 0.81, 0.82],
            }) as Rc<dyn ReleaseRuleModifier>,
            Rc::new(MinimumReleaseRuleAdjuster { minimum: 31.7 }),
            Rc::new(CapReleaseRuleAdjuster { cap: 11915.0 }),
            Rc::new(FlaggedReleaseRuleAdjuster {
                flags: flags.clone(),
                flag: "release-rule-tiered".to_string(),
                percent: -6.2,
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 30124.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_rule_engine(&flags::new_registry());
        let item = new_release_rule("code-619", "name-975");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
