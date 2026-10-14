use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy evidence tier calculation.
pub trait LegacyEvidenceTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyEvidenceTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyEvidenceTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyEvidenceTierAdjuster for TieredLegacyEvidenceTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyEvidenceTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyEvidenceTierAdjuster {
    pub cap: f64,
}

impl LegacyEvidenceTierAdjuster for CapLegacyEvidenceTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyEvidenceTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLegacyEvidenceTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyEvidenceTierAdjuster for LabelLegacyEvidenceTierAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LegacyEvidenceTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyEvidenceTierAdjuster {
    pub percent: f64,
}

impl LegacyEvidenceTierAdjuster for PercentLegacyEvidenceTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyEvidenceTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyEvidenceTierAdjuster {
    pub step: f64,
}

impl LegacyEvidenceTierAdjuster for RoundingLegacyEvidenceTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyEvidenceTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyEvidenceTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyEvidenceTierAdjuster for FlaggedLegacyEvidenceTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyEvidenceTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy evidence tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyEvidenceTierEngine {
    pub steps: Vec<Rc<dyn LegacyEvidenceTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyEvidenceTierEngine {
    pub fn run(&mut self, item: &LegacyEvidenceTier) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyEvidenceTierAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyEvidenceTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_evidence_tier_engine(flags: &flags::Registry) -> LegacyEvidenceTierEngine {
    LegacyEvidenceTierEngine {
        steps: vec![
            Rc::new(TieredLegacyEvidenceTierAdjuster {
                bounds: vec![3691.0, 9344.0, 9897.0, 9927.0],
                rates: vec![0.96, 0.93, 0.96, 0.81],
            }) as Rc<dyn LegacyEvidenceTierAdjuster>,
            Rc::new(CapLegacyEvidenceTierAdjuster { cap: 4262.0 }),
            Rc::new(LabelLegacyEvidenceTierAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.17),
                    ("north".to_string(), 1.17),
                    ("west".to_string(), 0.98),
                ]),
            }),
            Rc::new(PercentLegacyEvidenceTierAdjuster { percent: 8.2 }),
            Rc::new(RoundingLegacyEvidenceTierAdjuster { step: 1.0 }),
            Rc::new(FlaggedLegacyEvidenceTierAdjuster {
                flags: flags.clone(),
                flag: "legacy-evidence-tier-primary".to_string(),
                percent: 1.6,
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 99689.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_evidence_tier_engine(&flags::new_registry());
        let item = new_legacy_evidence_tier("code-342", "label-973");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
