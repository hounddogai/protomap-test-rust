use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the token summary calculation.
pub trait TokenSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenSummaryQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl TokenSummaryModifier for TokenSummaryQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &TokenSummary, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTokenSummaryAdjuster {
    pub step: f64,
}

impl TokenSummaryModifier for RoundingTokenSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TokenSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenSummaryModifier for FlaggedTokenSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenSummaryModifier for TieredTokenSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenSummaryAdjuster {
    pub minimum: f64,
}

impl TokenSummaryModifier for MinimumTokenSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenSummaryAdjuster {
    pub cap: f64,
}

impl TokenSummaryModifier for CapTokenSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the token summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenSummaryEngine {
    pub steps: Vec<Rc<dyn TokenSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenSummaryEngine {
    pub fn price(&mut self, item: &TokenSummary) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[TokenSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_token_summary_engine(flags: &flags::Registry) -> TokenSummaryEngine {
    TokenSummaryEngine {
        steps: vec![
            Rc::new(TokenSummaryQuotaUnitsOffsetAdjuster { weight: 0.8 }) as Rc<dyn TokenSummaryModifier>,
            Rc::new(RoundingTokenSummaryAdjuster { step: 0.5 }),
            Rc::new(FlaggedTokenSummaryAdjuster {
                flags: flags.clone(),
                flag: "token-summary-batched".to_string(),
                percent: 4.3,
            }),
            Rc::new(TieredTokenSummaryAdjuster {
                bounds: vec![522.0, 533.0, 9652.0],
                rates: vec![0.95, 0.82, 0.87],
            }),
            Rc::new(MinimumTokenSummaryAdjuster { minimum: 12.8 }),
            Rc::new(CapTokenSummaryAdjuster { cap: 29406.0 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 81286.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_summary_engine(&flags::new_registry());
        let item = new_token_summary("code-703", "name-6");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
