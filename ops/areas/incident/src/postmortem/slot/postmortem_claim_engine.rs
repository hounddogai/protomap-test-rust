use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the postmortem claim calculation.
pub trait PostmortemClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPostmortemClaimAdjuster {
    pub percent: f64,
}

impl PostmortemClaimFactor for PercentPostmortemClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PostmortemClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPostmortemClaimAdjuster {
    pub step: f64,
}

impl PostmortemClaimFactor for RoundingPostmortemClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PostmortemClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemClaimFactor for FlaggedPostmortemClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemClaimAdjuster {
    pub minimum: f64,
}

impl PostmortemClaimFactor for MinimumPostmortemClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PostmortemClaimFactor for TieredPostmortemClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PostmortemClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the postmortem claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemClaimEngine {
    pub steps: Vec<Rc<dyn PostmortemClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemClaimEngine {
    pub fn price(&mut self, item: &PostmortemClaim) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PostmortemClaimFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_postmortem_claim_engine(flags: &flags::Registry) -> PostmortemClaimEngine {
    PostmortemClaimEngine {
        steps: vec![
            Rc::new(PercentPostmortemClaimAdjuster { percent: 4.9 }) as Rc<dyn PostmortemClaimFactor>,
            Rc::new(RoundingPostmortemClaimAdjuster { step: 0.05 }),
            Rc::new(FlaggedPostmortemClaimAdjuster {
                flags: flags.clone(),
                flag: "postmortem-claim-seasonal".to_string(),
                percent: 0.5,
            }),
            Rc::new(MinimumPostmortemClaimAdjuster { minimum: 46.1 }),
            Rc::new(TieredPostmortemClaimAdjuster {
                bounds: vec![4888.0, 8382.0],
                rates: vec![1.01, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 93002.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_claim_engine(&flags::new_registry());
        let item = new_postmortem_claim("code-949", "label-988");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
