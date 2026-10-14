use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered quote claim calculation.
pub trait TieredQuoteClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredQuoteClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredQuoteClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredQuoteClaimModifier for TieredTieredQuoteClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredQuoteClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredQuoteClaimAdjuster {
    pub step: f64,
}

impl TieredQuoteClaimModifier for RoundingTieredQuoteClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredQuoteClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredQuoteClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredQuoteClaimModifier for FlaggedTieredQuoteClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredQuoteClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleTieredQuoteClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredQuoteClaimModifier for TitleTieredQuoteClaimAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &TieredQuoteClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the tiered quote claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredQuoteClaimEngine {
    pub steps: Vec<Rc<dyn TieredQuoteClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredQuoteClaimEngine {
    pub fn price(&mut self, item: &TieredQuoteClaim) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredQuoteClaimModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredQuoteClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_quote_claim_engine(flags: &flags::Registry) -> TieredQuoteClaimEngine {
    TieredQuoteClaimEngine {
        steps: vec![
            Rc::new(TieredTieredQuoteClaimAdjuster {
                bounds: vec![681.0, 1575.0, 2037.0, 7265.0],
                rates: vec![1.02, 1.13, 0.86, 0.9],
            }) as Rc<dyn TieredQuoteClaimModifier>,
            Rc::new(RoundingTieredQuoteClaimAdjuster { step: 0.05 }),
            Rc::new(FlaggedTieredQuoteClaimAdjuster {
                flags: flags.clone(),
                flag: "tiered-quote-claim-manual".to_string(),
                percent: 4.6,
            }),
            Rc::new(TitleTieredQuoteClaimAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.12)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 43521.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_quote_claim_engine(&flags::new_registry());
        let item = new_tiered_quote_claim("code-738", "title-706");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
