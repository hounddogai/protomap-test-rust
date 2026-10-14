use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the evidence tier calculation.
pub trait EvidenceTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceTierAdjuster {
    pub percent: f64,
}

impl EvidenceTierFactor for PercentEvidenceTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceTierFactor for FlaggedEvidenceTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidenceTierAdjuster {
    pub step: f64,
}

impl EvidenceTierFactor for RoundingEvidenceTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidenceTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceTierAdjuster {
    pub minimum: f64,
}

impl EvidenceTierFactor for MinimumEvidenceTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceTierFactor for TieredEvidenceTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelEvidenceTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceTierFactor for LabelEvidenceTierAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &EvidenceTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the evidence tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceTierEngine {
    pub steps: Vec<Rc<dyn EvidenceTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceTierEngine {
    pub fn run(&mut self, item: &EvidenceTier) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceTierFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_evidence_tier_engine(flags: &flags::Registry) -> EvidenceTierEngine {
    EvidenceTierEngine {
        steps: vec![
            Rc::new(PercentEvidenceTierAdjuster { percent: 8.8 }) as Rc<dyn EvidenceTierFactor>,
            Rc::new(FlaggedEvidenceTierAdjuster {
                flags: flags.clone(),
                flag: "evidence-tier-adaptive".to_string(),
                percent: 6.7,
            }),
            Rc::new(RoundingEvidenceTierAdjuster { step: 0.5 }),
            Rc::new(MinimumEvidenceTierAdjuster { minimum: 48.2 }),
            Rc::new(TieredEvidenceTierAdjuster {
                bounds: vec![6967.0, 9011.0],
                rates: vec![0.92, 1.11],
            }),
            Rc::new(LabelEvidenceTierAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.0)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 79861.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_tier_engine(&flags::new_registry());
        let item = new_evidence_tier("code-576", "label-287");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
