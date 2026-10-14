use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the token window calculation.
pub trait TokenWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTokenWindowAdjuster {
    pub step: f64,
}

impl TokenWindowModifier for RoundingTokenWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TokenWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenWindowModifier for FlaggedTokenWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenWindowAdjuster {
    pub minimum: f64,
}

impl TokenWindowModifier for MinimumTokenWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTokenWindowAdjuster {
    pub percent: f64,
}

impl TokenWindowModifier for PercentTokenWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TokenWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWindowTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl TokenWindowModifier for TokenWindowTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &TokenWindow, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

/// Runs the token window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenWindowEngine {
    pub steps: Vec<Rc<dyn TokenWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenWindowEngine {
    pub fn evaluate(&mut self, item: &TokenWindow) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TokenWindowModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TokenWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_token_window_engine(flags: &flags::Registry) -> TokenWindowEngine {
    TokenWindowEngine {
        steps: vec![
            Rc::new(RoundingTokenWindowAdjuster { step: 0.5 }) as Rc<dyn TokenWindowModifier>,
            Rc::new(FlaggedTokenWindowAdjuster {
                flags: flags.clone(),
                flag: "token-window-default".to_string(),
                percent: -6.0,
            }),
            Rc::new(MinimumTokenWindowAdjuster { minimum: 7.8 }),
            Rc::new(PercentTokenWindowAdjuster { percent: -10.6 }),
            Rc::new(TokenWindowTtlSecondsOffsetAdjuster { weight: 1.78 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 53810.4,
    }
}
