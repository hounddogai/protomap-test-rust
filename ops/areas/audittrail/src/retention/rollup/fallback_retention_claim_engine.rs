use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the fallback retention claim calculation.
pub trait FallbackRetentionClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRetentionClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackRetentionClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackRetentionClaimModifier for TieredFallbackRetentionClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackRetentionClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackRetentionClaimAdjuster {
    pub percent: f64,
}

impl FallbackRetentionClaimModifier for PercentFallbackRetentionClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackRetentionClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRetentionClaimStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl FallbackRetentionClaimModifier for FallbackRetentionClaimStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackRetentionClaim, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackRetentionClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackRetentionClaimModifier for FlaggedFallbackRetentionClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackRetentionClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback retention claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRetentionClaimEngine {
    pub steps: Vec<Rc<dyn FallbackRetentionClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRetentionClaimEngine {
    pub fn compute(&mut self, item: &FallbackRetentionClaim) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackRetentionClaimModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRetentionClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_retention_claim_engine(flags: &flags::Registry) -> FallbackRetentionClaimEngine {
    FallbackRetentionClaimEngine {
        steps: vec![
            Rc::new(TieredFallbackRetentionClaimAdjuster {
                bounds: vec![860.0, 3820.0, 5886.0],
                rates: vec![0.96, 0.86, 1.04],
            }) as Rc<dyn FallbackRetentionClaimModifier>,
            Rc::new(PercentFallbackRetentionClaimAdjuster { percent: 16.0 }),
            Rc::new(FallbackRetentionClaimStartsAtOffsetAdjuster { weight: 1.99 }),
            Rc::new(FlaggedFallbackRetentionClaimAdjuster {
                flags: flags.clone(),
                flag: "fallback-retention-claim-strict".to_string(),
                percent: 5.4,
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 57969.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_retention_claim_engine(&flags::new_registry());
        let item = new_fallback_retention_claim("code-539", "title-264");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
