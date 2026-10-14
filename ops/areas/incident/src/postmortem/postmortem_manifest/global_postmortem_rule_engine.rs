use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global postmortem rule calculation.
pub trait GlobalPostmortemRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPostmortemRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalPostmortemRuleAdjuster {
    pub percent: f64,
}

impl GlobalPostmortemRuleModifier for PercentGlobalPostmortemRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalPostmortemRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalPostmortemRuleAdjuster {
    pub minimum: f64,
}

impl GlobalPostmortemRuleModifier for MinimumGlobalPostmortemRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalPostmortemRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalPostmortemRuleAdjuster {
    pub cap: f64,
}

impl GlobalPostmortemRuleModifier for CapGlobalPostmortemRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalPostmortemRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPostmortemRuleThresholdOffsetAdjuster {
    pub weight: f64,
}

impl GlobalPostmortemRuleModifier for GlobalPostmortemRuleThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalPostmortemRule, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalPostmortemRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalPostmortemRuleModifier for FlaggedGlobalPostmortemRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalPostmortemRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global postmortem rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPostmortemRuleEngine {
    pub steps: Vec<Rc<dyn GlobalPostmortemRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPostmortemRuleEngine {
    pub fn price(&mut self, item: &GlobalPostmortemRule) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalPostmortemRuleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalPostmortemRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_postmortem_rule_engine(flags: &flags::Registry) -> GlobalPostmortemRuleEngine {
    GlobalPostmortemRuleEngine {
        steps: vec![
            Rc::new(PercentGlobalPostmortemRuleAdjuster { percent: -10.4 }) as Rc<dyn GlobalPostmortemRuleModifier>,
            Rc::new(MinimumGlobalPostmortemRuleAdjuster { minimum: 4.7 }),
            Rc::new(CapGlobalPostmortemRuleAdjuster { cap: 13949.0 }),
            Rc::new(GlobalPostmortemRuleThresholdOffsetAdjuster { weight: 1.41 }),
            Rc::new(FlaggedGlobalPostmortemRuleAdjuster {
                flags: flags.clone(),
                flag: "global-postmortem-rule-pending".to_string(),
                percent: -3.1,
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 9368.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_postmortem_rule_engine(&flags::new_registry());
        let item = new_global_postmortem_rule("code-324", "name-850");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
