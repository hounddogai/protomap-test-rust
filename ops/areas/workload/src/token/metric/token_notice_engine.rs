use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the token notice calculation.
pub trait TokenNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenNoticeAdjuster {
    pub minimum: f64,
}

impl TokenNoticeStep for MinimumTokenNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTokenNoticeAdjuster {
    pub percent: f64,
}

impl TokenNoticeStep for PercentTokenNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TokenNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenNoticeStep for FlaggedTokenNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the token notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenNoticeEngine {
    pub steps: Vec<Rc<dyn TokenNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenNoticeEngine {
    pub fn run(&mut self, item: &TokenNotice) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TokenNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TokenNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_token_notice_engine(flags: &flags::Registry) -> TokenNoticeEngine {
    TokenNoticeEngine {
        steps: vec![
            Rc::new(MinimumTokenNoticeAdjuster { minimum: 31.6 }) as Rc<dyn TokenNoticeStep>,
            Rc::new(PercentTokenNoticeAdjuster { percent: -22.7 }),
            Rc::new(FlaggedTokenNoticeAdjuster {
                flags: flags.clone(),
                flag: "token-notice-global".to_string(),
                percent: 3.7,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 78141.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_notice_engine(&flags::new_registry());
        let item = new_token_notice("code-817", "title-963");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
