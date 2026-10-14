use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the token ledger calculation.
pub trait TokenLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenLedgerRatioOffsetAdjuster {
    pub weight: f64,
}

impl TokenLedgerStep for TokenLedgerRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &TokenLedger, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenLedgerAdjuster {
    pub cap: f64,
}

impl TokenLedgerStep for CapTokenLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenLedgerAdjuster {
    pub minimum: f64,
}

impl TokenLedgerStep for MinimumTokenLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenLedgerStep for FlaggedTokenLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterTokenLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenLedgerStep for ClusterTokenLedgerAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &TokenLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the token ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenLedgerEngine {
    pub steps: Vec<Rc<dyn TokenLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenLedgerEngine {
    pub fn compute(&mut self, item: &TokenLedger) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TokenLedgerStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TokenLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_token_ledger_engine(flags: &flags::Registry) -> TokenLedgerEngine {
    TokenLedgerEngine {
        steps: vec![
            Rc::new(TokenLedgerRatioOffsetAdjuster { weight: 0.84 }) as Rc<dyn TokenLedgerStep>,
            Rc::new(CapTokenLedgerAdjuster { cap: 32522.0 }),
            Rc::new(MinimumTokenLedgerAdjuster { minimum: 42.5 }),
            Rc::new(FlaggedTokenLedgerAdjuster {
                flags: flags.clone(),
                flag: "token-ledger-scheduled".to_string(),
                percent: 1.8,
            }),
            Rc::new(ClusterTokenLedgerAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.26),
                    ("retail".to_string(), 1.39),
                    ("online".to_string(), 0.88),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 4798.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_ledger_engine(&flags::new_registry());
        let item = new_token_ledger("code-376", "name-262");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
