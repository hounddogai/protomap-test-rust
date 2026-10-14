use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary dns record tier calculation.
pub trait PrimaryDnsRecordTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryDnsRecordTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryDnsRecordTierAdjuster {
    pub cap: f64,
}

impl PrimaryDnsRecordTierModifier for CapPrimaryDnsRecordTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryDnsRecordTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPrimaryDnsRecordTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryDnsRecordTierModifier for LabelPrimaryDnsRecordTierAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PrimaryDnsRecordTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryDnsRecordTierAdjuster {
    pub percent: f64,
}

impl PrimaryDnsRecordTierModifier for PercentPrimaryDnsRecordTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryDnsRecordTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryDnsRecordTierAdjuster {
    pub step: f64,
}

impl PrimaryDnsRecordTierModifier for RoundingPrimaryDnsRecordTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryDnsRecordTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordTierMarginOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryDnsRecordTierModifier for PrimaryDnsRecordTierMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryDnsRecordTier, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryDnsRecordTierAdjuster {
    pub minimum: f64,
}

impl PrimaryDnsRecordTierModifier for MinimumPrimaryDnsRecordTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryDnsRecordTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the primary dns record tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryDnsRecordTierEngine {
    pub steps: Vec<Rc<dyn PrimaryDnsRecordTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryDnsRecordTierEngine {
    pub fn price(&mut self, item: &PrimaryDnsRecordTier) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryDnsRecordTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_dns_record_tier_engine() -> PrimaryDnsRecordTierEngine {
    PrimaryDnsRecordTierEngine {
        steps: vec![
            Rc::new(CapPrimaryDnsRecordTierAdjuster { cap: 6885.0 }) as Rc<dyn PrimaryDnsRecordTierModifier>,
            Rc::new(LabelPrimaryDnsRecordTierAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.83)]),
            }),
            Rc::new(PercentPrimaryDnsRecordTierAdjuster { percent: 21.1 }),
            Rc::new(RoundingPrimaryDnsRecordTierAdjuster { step: 5.0 }),
            Rc::new(PrimaryDnsRecordTierMarginOffsetAdjuster { weight: 1.57 }),
            Rc::new(MinimumPrimaryDnsRecordTierAdjuster { minimum: 25.3 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 94388.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_dns_record_tier_engine();
        let item = new_primary_dns_record_tier("code-844", "label-992");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
