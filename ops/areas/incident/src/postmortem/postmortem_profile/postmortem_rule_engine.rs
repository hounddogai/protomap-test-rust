use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the postmortem rule calculation.
pub trait PostmortemRuleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPostmortemRuleAdjuster {
    pub step: f64,
}

impl PostmortemRuleAdjuster for RoundingPostmortemRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PostmortemRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRuleExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemRuleAdjuster for PostmortemRuleExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemRule, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PostmortemRuleAdjuster for TieredPostmortemRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PostmortemRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemRuleAdjuster for FlaggedPostmortemRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the postmortem rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemRuleEngine {
    pub steps: Vec<Rc<dyn PostmortemRuleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemRuleEngine {
    pub fn run(&mut self, item: &PostmortemRule) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PostmortemRuleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_postmortem_rule_engine(flags: &flags::Registry) -> PostmortemRuleEngine {
    PostmortemRuleEngine {
        steps: vec![
            Rc::new(RoundingPostmortemRuleAdjuster { step: 0.25 }) as Rc<dyn PostmortemRuleAdjuster>,
            Rc::new(PostmortemRuleExpiresAtOffsetAdjuster { weight: 0.33 }),
            Rc::new(TieredPostmortemRuleAdjuster {
                bounds: vec![7758.0, 8670.0],
                rates: vec![1.1, 0.87],
            }),
            Rc::new(FlaggedPostmortemRuleAdjuster {
                flags: flags.clone(),
                flag: "postmortem-rule-strict".to_string(),
                percent: 1.2,
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 41120.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_rule_engine(&flags::new_registry());
        let item = new_postmortem_rule("code-84", "label-932");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
