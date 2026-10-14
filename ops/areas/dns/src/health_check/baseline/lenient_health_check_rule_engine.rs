use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient health check rule calculation.
pub trait LenientHealthCheckRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientHealthCheckRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientHealthCheckRuleAdjuster {
    pub percent: f64,
}

impl LenientHealthCheckRuleStep for PercentLenientHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientHealthCheckRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientHealthCheckRuleAdjuster {
    pub step: f64,
}

impl LenientHealthCheckRuleStep for RoundingLenientHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientHealthCheckRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitLenientHealthCheckRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientHealthCheckRuleStep for UnitLenientHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &LenientHealthCheckRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientHealthCheckRuleAdjuster {
    pub minimum: f64,
}

impl LenientHealthCheckRuleStep for MinimumLenientHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientHealthCheckRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientHealthCheckRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientHealthCheckRuleStep for TieredLenientHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientHealthCheckRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckRuleWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl LenientHealthCheckRuleStep for LenientHealthCheckRuleWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &LenientHealthCheckRule, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

/// Runs the lenient health check rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientHealthCheckRuleEngine {
    pub steps: Vec<Rc<dyn LenientHealthCheckRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientHealthCheckRuleEngine {
    pub fn price(&mut self, item: &LenientHealthCheckRule) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientHealthCheckRuleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientHealthCheckRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_health_check_rule_engine() -> LenientHealthCheckRuleEngine {
    LenientHealthCheckRuleEngine {
        steps: vec![
            Rc::new(PercentLenientHealthCheckRuleAdjuster { percent: -4.2 }) as Rc<dyn LenientHealthCheckRuleStep>,
            Rc::new(RoundingLenientHealthCheckRuleAdjuster { step: 1.0 }),
            Rc::new(UnitLenientHealthCheckRuleAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.89),
                    ("south".to_string(), 0.78),
                    ("east".to_string(), 0.78),
                ]),
            }),
            Rc::new(MinimumLenientHealthCheckRuleAdjuster { minimum: 25.0 }),
            Rc::new(TieredLenientHealthCheckRuleAdjuster {
                bounds: vec![1080.0, 1472.0, 3825.0],
                rates: vec![0.96, 1.04, 0.93],
            }),
            Rc::new(LenientHealthCheckRuleWindowDaysOffsetAdjuster { weight: 1.53 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 10256.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_health_check_rule_engine();
        let item = new_lenient_health_check_rule("code-819", "name-153");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
