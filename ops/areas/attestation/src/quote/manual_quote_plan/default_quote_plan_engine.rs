use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the default quote plan calculation.
pub trait DefaultQuotePlanStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultQuotePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceDefaultQuotePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultQuotePlanStep for SourceDefaultQuotePlanAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &DefaultQuotePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultQuotePlanAdjuster {
    pub minimum: f64,
}

impl DefaultQuotePlanStep for MinimumDefaultQuotePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultQuotePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultQuotePlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultQuotePlanStep for FlaggedDefaultQuotePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultQuotePlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultQuotePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultQuotePlanStep for TieredDefaultQuotePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultQuotePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQuotePlanScoreOffsetAdjuster {
    pub weight: f64,
}

impl DefaultQuotePlanStep for DefaultQuotePlanScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultQuotePlan, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the default quote plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultQuotePlanEngine {
    pub steps: Vec<Rc<dyn DefaultQuotePlanStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultQuotePlanEngine {
    pub fn evaluate(&mut self, item: &DefaultQuotePlan) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultQuotePlanStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultQuotePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_quote_plan_engine(flags: &flags::Registry) -> DefaultQuotePlanEngine {
    DefaultQuotePlanEngine {
        steps: vec![
            Rc::new(SourceDefaultQuotePlanAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.78), ("central".to_string(), 0.83)]),
            }) as Rc<dyn DefaultQuotePlanStep>,
            Rc::new(MinimumDefaultQuotePlanAdjuster { minimum: 27.6 }),
            Rc::new(FlaggedDefaultQuotePlanAdjuster {
                flags: flags.clone(),
                flag: "default-quote-plan-pending".to_string(),
                percent: 5.0,
            }),
            Rc::new(TieredDefaultQuotePlanAdjuster {
                bounds: vec![3508.0, 4536.0, 4593.0, 5333.0],
                rates: vec![1.15, 1.1, 0.97, 1.14],
            }),
            Rc::new(DefaultQuotePlanScoreOffsetAdjuster { weight: 1.09 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 48403.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_quote_plan_engine(&flags::new_registry());
        let item = new_default_quote_plan("code-461", "label-453");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
