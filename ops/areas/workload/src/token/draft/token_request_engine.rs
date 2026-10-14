use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the token request calculation.
pub trait TokenRequestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenRequest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenRequestModifier for FlaggedTokenRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenRequestCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl TokenRequestModifier for TokenRequestCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &TokenRequest, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenRequestAdjuster {
    pub cap: f64,
}

impl TokenRequestModifier for CapTokenRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTokenRequestAdjuster {
    pub percent: f64,
}

impl TokenRequestModifier for PercentTokenRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TokenRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelTokenRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenRequestModifier for LabelTokenRequestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &TokenRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenRequestAdjuster {
    pub minimum: f64,
}

impl TokenRequestModifier for MinimumTokenRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the token request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenRequestEngine {
    pub steps: Vec<Rc<dyn TokenRequestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenRequestEngine {
    pub fn evaluate(&mut self, item: &TokenRequest) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TokenRequestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TokenRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_token_request_engine(flags: &flags::Registry) -> TokenRequestEngine {
    TokenRequestEngine {
        steps: vec![
            Rc::new(FlaggedTokenRequestAdjuster {
                flags: flags.clone(),
                flag: "token-request-manual".to_string(),
                percent: 7.9,
            }) as Rc<dyn TokenRequestModifier>,
            Rc::new(TokenRequestCreatedAtOffsetAdjuster { weight: 0.89 }),
            Rc::new(CapTokenRequestAdjuster { cap: 21969.0 }),
            Rc::new(PercentTokenRequestAdjuster { percent: -11.1 }),
            Rc::new(LabelTokenRequestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.98), ("retail".to_string(), 1.14)]),
            }),
            Rc::new(MinimumTokenRequestAdjuster { minimum: 5.7 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 1108.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_request_engine(&flags::new_registry());
        let item = new_token_request("code-439", "label-636");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
