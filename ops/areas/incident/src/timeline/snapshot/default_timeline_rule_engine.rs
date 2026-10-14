use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the default timeline rule calculation.
pub trait DefaultTimelineRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultTimelineRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDefaultTimelineRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultTimelineRuleFactor for NameDefaultTimelineRuleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DefaultTimelineRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultTimelineRuleAdjuster {
    pub minimum: f64,
}

impl DefaultTimelineRuleFactor for MinimumDefaultTimelineRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultTimelineRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultTimelineRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultTimelineRuleFactor for TieredDefaultTimelineRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultTimelineRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultTimelineRuleAdjuster {
    pub cap: f64,
}

impl DefaultTimelineRuleFactor for CapDefaultTimelineRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultTimelineRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultTimelineRuleAdjuster {
    pub percent: f64,
}

impl DefaultTimelineRuleFactor for PercentDefaultTimelineRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultTimelineRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default timeline rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultTimelineRuleEngine {
    pub steps: Vec<Rc<dyn DefaultTimelineRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultTimelineRuleEngine {
    pub fn price(&mut self, item: &DefaultTimelineRule) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultTimelineRuleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultTimelineRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_timeline_rule_engine() -> DefaultTimelineRuleEngine {
    DefaultTimelineRuleEngine {
        steps: vec![
            Rc::new(NameDefaultTimelineRuleAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.31),
                    ("retail".to_string(), 1.26),
                    ("west".to_string(), 0.9),
                ]),
            }) as Rc<dyn DefaultTimelineRuleFactor>,
            Rc::new(MinimumDefaultTimelineRuleAdjuster { minimum: 34.4 }),
            Rc::new(TieredDefaultTimelineRuleAdjuster {
                bounds: vec![648.0, 1271.0, 8564.0, 9541.0],
                rates: vec![1.04, 1.0, 0.84, 0.96],
            }),
            Rc::new(CapDefaultTimelineRuleAdjuster { cap: 7920.0 }),
            Rc::new(PercentDefaultTimelineRuleAdjuster { percent: 16.8 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 13926.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_timeline_rule_engine();
        let item = new_default_timeline_rule("code-339", "name-184");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
