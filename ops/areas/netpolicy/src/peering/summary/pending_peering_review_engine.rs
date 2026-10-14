use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the pending peering review calculation.
pub trait PendingPeeringReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingPeeringReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingPeeringReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingPeeringReviewModifier for TieredPendingPeeringReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingPeeringReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingPeeringReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingPeeringReviewModifier for FlaggedPendingPeeringReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingPeeringReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingPeeringReviewAdjuster {
    pub minimum: f64,
}

impl PendingPeeringReviewModifier for MinimumPendingPeeringReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingPeeringReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending peering review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingPeeringReviewEngine {
    pub steps: Vec<Rc<dyn PendingPeeringReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingPeeringReviewEngine {
    pub fn compute(&mut self, item: &PendingPeeringReview) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingPeeringReviewModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingPeeringReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_peering_review_engine(flags: &flags::Registry) -> PendingPeeringReviewEngine {
    PendingPeeringReviewEngine {
        steps: vec![
            Rc::new(TieredPendingPeeringReviewAdjuster {
                bounds: vec![1148.0, 3132.0, 5057.0],
                rates: vec![1.15, 0.82, 1.12],
            }) as Rc<dyn PendingPeeringReviewModifier>,
            Rc::new(FlaggedPendingPeeringReviewAdjuster {
                flags: flags.clone(),
                flag: "pending-peering-review-seasonal".to_string(),
                percent: 5.0,
            }),
            Rc::new(MinimumPendingPeeringReviewAdjuster { minimum: 18.1 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 85889.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_peering_review_engine(&flags::new_registry());
        let item = new_pending_peering_review("code-843", "title-490");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
