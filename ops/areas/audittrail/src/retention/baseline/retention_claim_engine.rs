use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention claim calculation.
pub trait RetentionClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionClaimAdjuster {
    pub cap: f64,
}

impl RetentionClaimStep for CapRetentionClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionClaimAdjuster {
    pub percent: f64,
}

impl RetentionClaimStep for PercentRetentionClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionClaimAdjuster {
    pub step: f64,
}

impl RetentionClaimStep for RoundingRetentionClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRetentionClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionClaimStep for TitleRetentionClaimAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RetentionClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionClaimStep for FlaggedRetentionClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the retention claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionClaimEngine {
    pub steps: Vec<Rc<dyn RetentionClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionClaimEngine {
    pub fn run(&mut self, item: &RetentionClaim) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionClaimStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_retention_claim_engine(flags: &flags::Registry) -> RetentionClaimEngine {
    RetentionClaimEngine {
        steps: vec![
            Rc::new(CapRetentionClaimAdjuster { cap: 19307.0 }) as Rc<dyn RetentionClaimStep>,
            Rc::new(PercentRetentionClaimAdjuster { percent: -20.1 }),
            Rc::new(RoundingRetentionClaimAdjuster { step: 1.0 }),
            Rc::new(TitleRetentionClaimAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.23)]),
            }),
            Rc::new(FlaggedRetentionClaimAdjuster {
                flags: flags.clone(),
                flag: "retention-claim-strict".to_string(),
                percent: -5.9,
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 52291.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_claim_engine(&flags::new_registry());
        let item = new_retention_claim("code-338", "title-994");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
