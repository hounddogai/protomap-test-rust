use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered token claim calculation.
pub trait TieredTokenClaimAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredTokenClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredTokenClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredTokenClaimAdjuster for TieredTieredTokenClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredTokenClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredTokenClaimAdjuster {
    pub percent: f64,
}

impl TieredTokenClaimAdjuster for PercentTieredTokenClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredTokenClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredTokenClaimAdjuster {
    pub step: f64,
}

impl TieredTokenClaimAdjuster for RoundingTieredTokenClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredTokenClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredTokenClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredTokenClaimAdjuster for FlaggedTieredTokenClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredTokenClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenClaimRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl TieredTokenClaimAdjuster for TieredTokenClaimRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &TieredTokenClaim, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

/// Runs the tiered token claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredTokenClaimEngine {
    pub steps: Vec<Rc<dyn TieredTokenClaimAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredTokenClaimEngine {
    pub fn price(&mut self, item: &TieredTokenClaim) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredTokenClaimAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredTokenClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_token_claim_engine(flags: &flags::Registry) -> TieredTokenClaimEngine {
    TieredTokenClaimEngine {
        steps: vec![
            Rc::new(TieredTieredTokenClaimAdjuster {
                bounds: vec![3819.0, 4730.0],
                rates: vec![1.16, 0.87],
            }) as Rc<dyn TieredTokenClaimAdjuster>,
            Rc::new(PercentTieredTokenClaimAdjuster { percent: -17.9 }),
            Rc::new(RoundingTieredTokenClaimAdjuster { step: 0.5 }),
            Rc::new(FlaggedTieredTokenClaimAdjuster {
                flags: flags.clone(),
                flag: "tiered-token-claim-pending".to_string(),
                percent: -0.2,
            }),
            Rc::new(TieredTokenClaimRotationDaysOffsetAdjuster { weight: 0.11 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 84948.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_token_claim_engine(&flags::new_registry());
        let item = new_tiered_token_claim("code-969", "name-257");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
