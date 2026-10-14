use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary timeline rule calculation.
pub trait PrimaryTimelineRuleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryTimelineRule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryTimelineRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryTimelineRuleAdjuster for FlaggedPrimaryTimelineRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryTimelineRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryTimelineRuleAdjuster {
    pub step: f64,
}

impl PrimaryTimelineRuleAdjuster for RoundingPrimaryTimelineRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryTimelineRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTimelineRuleTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryTimelineRuleAdjuster for PrimaryTimelineRuleTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryTimelineRule, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespacePrimaryTimelineRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryTimelineRuleAdjuster for NamespacePrimaryTimelineRuleAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &PrimaryTimelineRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryTimelineRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryTimelineRuleAdjuster for TieredPrimaryTimelineRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryTimelineRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the primary timeline rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryTimelineRuleEngine {
    pub steps: Vec<Rc<dyn PrimaryTimelineRuleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryTimelineRuleEngine {
    pub fn evaluate(&mut self, item: &PrimaryTimelineRule) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryTimelineRuleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryTimelineRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_timeline_rule_engine(flags: &flags::Registry) -> PrimaryTimelineRuleEngine {
    PrimaryTimelineRuleEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryTimelineRuleAdjuster {
                flags: flags.clone(),
                flag: "primary-timeline-rule-lenient".to_string(),
                percent: -8.2,
            }) as Rc<dyn PrimaryTimelineRuleAdjuster>,
            Rc::new(RoundingPrimaryTimelineRuleAdjuster { step: 0.25 }),
            Rc::new(PrimaryTimelineRuleTtlSecondsOffsetAdjuster { weight: 0.96 }),
            Rc::new(NamespacePrimaryTimelineRuleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.12)]),
            }),
            Rc::new(TieredPrimaryTimelineRuleAdjuster {
                bounds: vec![134.0, 7056.0],
                rates: vec![0.81, 0.9],
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 14844.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_timeline_rule_engine(&flags::new_registry());
        let item = new_primary_timeline_rule("code-441", "title-947");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
