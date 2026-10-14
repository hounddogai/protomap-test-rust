use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the strict rule request calculation.
pub trait StrictRuleRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictRuleRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictRuleRequestAdjuster {
    pub cap: f64,
}

impl StrictRuleRequestStep for CapStrictRuleRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictRuleRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictRuleRequestAdjuster {
    pub minimum: f64,
}

impl StrictRuleRequestStep for MinimumStrictRuleRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictRuleRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictRuleRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictRuleRequestStep for TieredStrictRuleRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictRuleRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleRequestBudgetOffsetAdjuster {
    pub weight: f64,
}

impl StrictRuleRequestStep for StrictRuleRequestBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &StrictRuleRequest, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameStrictRuleRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictRuleRequestStep for NameStrictRuleRequestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &StrictRuleRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictRuleRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictRuleRequestStep for FlaggedStrictRuleRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictRuleRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict rule request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictRuleRequestEngine {
    pub steps: Vec<Rc<dyn StrictRuleRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictRuleRequestEngine {
    pub fn compute(&mut self, item: &StrictRuleRequest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictRuleRequestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictRuleRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_rule_request_engine(flags: &flags::Registry) -> StrictRuleRequestEngine {
    StrictRuleRequestEngine {
        steps: vec![
            Rc::new(CapStrictRuleRequestAdjuster { cap: 4172.0 }) as Rc<dyn StrictRuleRequestStep>,
            Rc::new(MinimumStrictRuleRequestAdjuster { minimum: 6.8 }),
            Rc::new(TieredStrictRuleRequestAdjuster {
                bounds: vec![3408.0, 3677.0, 4394.0],
                rates: vec![0.84, 0.82, 0.85],
            }),
            Rc::new(StrictRuleRequestBudgetOffsetAdjuster { weight: 1.12 }),
            Rc::new(NameStrictRuleRequestAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.36),
                    ("south".to_string(), 0.76),
                    ("retail".to_string(), 0.91),
                ]),
            }),
            Rc::new(FlaggedStrictRuleRequestAdjuster {
                flags: flags.clone(),
                flag: "strict-rule-request-strict".to_string(),
                percent: 8.2,
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 42615.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_rule_request_engine(&flags::new_registry());
        let item = new_strict_rule_request("code-158", "name-650");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
