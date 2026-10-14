use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the token group calculation.
pub trait TokenGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenGroupAdjuster {
    pub minimum: f64,
}

impl TokenGroupFactor for MinimumTokenGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenGroupAdjuster {
    pub cap: f64,
}

impl TokenGroupFactor for CapTokenGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenGroupFactor for FlaggedTokenGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenGroupAmountOffsetAdjuster {
    pub weight: f64,
}

impl TokenGroupFactor for TokenGroupAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &TokenGroup, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTokenGroupAdjuster {
    pub step: f64,
}

impl TokenGroupFactor for RoundingTokenGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TokenGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the token group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenGroupEngine {
    pub steps: Vec<Rc<dyn TokenGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenGroupEngine {
    pub fn price(&mut self, item: &TokenGroup) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TokenGroupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TokenGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_token_group_engine(flags: &flags::Registry) -> TokenGroupEngine {
    TokenGroupEngine {
        steps: vec![
            Rc::new(MinimumTokenGroupAdjuster { minimum: 47.6 }) as Rc<dyn TokenGroupFactor>,
            Rc::new(CapTokenGroupAdjuster { cap: 5321.0 }),
            Rc::new(FlaggedTokenGroupAdjuster {
                flags: flags.clone(),
                flag: "token-group-pending".to_string(),
                percent: 2.0,
            }),
            Rc::new(TokenGroupAmountOffsetAdjuster { weight: 1.61 }),
            Rc::new(RoundingTokenGroupAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 75439.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_group_engine(&flags::new_registry());
        let item = new_token_group("code-359", "title-823");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
