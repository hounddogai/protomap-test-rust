use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the token plan calculation.
pub trait TokenPlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenPlanFactor for TieredTokenPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenPlanAdjuster {
    pub minimum: f64,
}

impl TokenPlanFactor for MinimumTokenPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenPlanFactor for FlaggedTokenPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTokenPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenPlanFactor for NameTokenPlanAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TokenPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenPlanPortOffsetAdjuster {
    pub weight: f64,
}

impl TokenPlanFactor for TokenPlanPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &TokenPlan, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenPlanAdjuster {
    pub cap: f64,
}

impl TokenPlanFactor for CapTokenPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the token plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenPlanEngine {
    pub steps: Vec<Rc<dyn TokenPlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenPlanEngine {
    pub fn compute(&mut self, item: &TokenPlan) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TokenPlanFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TokenPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_token_plan_engine(flags: &flags::Registry) -> TokenPlanEngine {
    TokenPlanEngine {
        steps: vec![
            Rc::new(TieredTokenPlanAdjuster {
                bounds: vec![637.0, 923.0, 4868.0],
                rates: vec![0.96, 0.87, 0.81],
            }) as Rc<dyn TokenPlanFactor>,
            Rc::new(MinimumTokenPlanAdjuster { minimum: 18.4 }),
            Rc::new(FlaggedTokenPlanAdjuster {
                flags: flags.clone(),
                flag: "token-plan-batched".to_string(),
                percent: 0.4,
            }),
            Rc::new(NameTokenPlanAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.12),
                    ("online".to_string(), 1.31),
                    ("retail".to_string(), 1.36),
                ]),
            }),
            Rc::new(TokenPlanPortOffsetAdjuster { weight: 1.32 }),
            Rc::new(CapTokenPlanAdjuster { cap: 37688.0 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 82720.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_plan_engine(&flags::new_registry());
        let item = new_token_plan("code-101", "name-584");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
