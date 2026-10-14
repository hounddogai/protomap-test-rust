use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tiered segment claim calculation.
pub trait TieredSegmentClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredSegmentClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredSegmentClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredSegmentClaimFactor for TieredTieredSegmentClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredSegmentClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredSegmentClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredSegmentClaimFactor for FlaggedTieredSegmentClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredSegmentClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleTieredSegmentClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredSegmentClaimFactor for LocaleTieredSegmentClaimAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &TieredSegmentClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the tiered segment claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredSegmentClaimEngine {
    pub steps: Vec<Rc<dyn TieredSegmentClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredSegmentClaimEngine {
    pub fn evaluate(&mut self, item: &TieredSegmentClaim) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredSegmentClaimFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredSegmentClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_segment_claim_engine(flags: &flags::Registry) -> TieredSegmentClaimEngine {
    TieredSegmentClaimEngine {
        steps: vec![
            Rc::new(TieredTieredSegmentClaimAdjuster {
                bounds: vec![327.0, 428.0, 2073.0],
                rates: vec![0.81, 0.89, 1.04],
            }) as Rc<dyn TieredSegmentClaimFactor>,
            Rc::new(FlaggedTieredSegmentClaimAdjuster {
                flags: flags.clone(),
                flag: "tiered-segment-claim-fallback".to_string(),
                percent: -0.4,
            }),
            Rc::new(LocaleTieredSegmentClaimAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.83)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 84507.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_segment_claim_engine(&flags::new_registry());
        let item = new_tiered_segment_claim("code-227", "name-510");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
