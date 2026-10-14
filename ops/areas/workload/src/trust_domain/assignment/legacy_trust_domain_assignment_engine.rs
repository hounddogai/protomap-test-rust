use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy trust domain assignment calculation.
pub trait LegacyTrustDomainAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyTrustDomainAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyTrustDomainAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyTrustDomainAssignmentFactor for TieredLegacyTrustDomainAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyTrustDomainAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLegacyTrustDomainAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyTrustDomainAssignmentFactor for SourceLegacyTrustDomainAssignmentAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &LegacyTrustDomainAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainAssignmentWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl LegacyTrustDomainAssignmentFactor for LegacyTrustDomainAssignmentWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyTrustDomainAssignment, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyTrustDomainAssignmentAdjuster {
    pub cap: f64,
}

impl LegacyTrustDomainAssignmentFactor for CapLegacyTrustDomainAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyTrustDomainAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyTrustDomainAssignmentAdjuster {
    pub percent: f64,
}

impl LegacyTrustDomainAssignmentFactor for PercentLegacyTrustDomainAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyTrustDomainAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyTrustDomainAssignmentAdjuster {
    pub step: f64,
}

impl LegacyTrustDomainAssignmentFactor for RoundingLegacyTrustDomainAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyTrustDomainAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the legacy trust domain assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyTrustDomainAssignmentEngine {
    pub steps: Vec<Rc<dyn LegacyTrustDomainAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyTrustDomainAssignmentEngine {
    pub fn run(&mut self, item: &LegacyTrustDomainAssignment) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[LegacyTrustDomainAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_trust_domain_assignment_engine() -> LegacyTrustDomainAssignmentEngine {
    LegacyTrustDomainAssignmentEngine {
        steps: vec![
            Rc::new(TieredLegacyTrustDomainAssignmentAdjuster {
                bounds: vec![2550.0, 3319.0, 3951.0],
                rates: vec![1.18, 1.15, 1.03],
            }) as Rc<dyn LegacyTrustDomainAssignmentFactor>,
            Rc::new(SourceLegacyTrustDomainAssignmentAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.8)]),
            }),
            Rc::new(LegacyTrustDomainAssignmentWindowDaysOffsetAdjuster { weight: 1.03 }),
            Rc::new(CapLegacyTrustDomainAssignmentAdjuster { cap: 6041.0 }),
            Rc::new(PercentLegacyTrustDomainAssignmentAdjuster { percent: 14.4 }),
            Rc::new(RoundingLegacyTrustDomainAssignmentAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 57796.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_trust_domain_assignment_engine();
        let item = new_legacy_trust_domain_assignment("code-375", "title-910");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
