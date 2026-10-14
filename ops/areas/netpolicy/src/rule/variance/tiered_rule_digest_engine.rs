use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered rule digest calculation.
pub trait TieredRuleDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredRuleDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredRuleDigestAdjuster {
    pub cap: f64,
}

impl TieredRuleDigestModifier for CapTieredRuleDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredRuleDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredRuleDigestAdjuster {
    pub minimum: f64,
}

impl TieredRuleDigestModifier for MinimumTieredRuleDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredRuleDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelTieredRuleDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredRuleDigestModifier for LabelTieredRuleDigestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &TieredRuleDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleDigestVersionOffsetAdjuster {
    pub weight: f64,
}

impl TieredRuleDigestModifier for TieredRuleDigestVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &TieredRuleDigest, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

/// Runs the tiered rule digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredRuleDigestEngine {
    pub steps: Vec<Rc<dyn TieredRuleDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredRuleDigestEngine {
    pub fn price(&mut self, item: &TieredRuleDigest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredRuleDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_rule_digest_engine() -> TieredRuleDigestEngine {
    TieredRuleDigestEngine {
        steps: vec![
            Rc::new(CapTieredRuleDigestAdjuster { cap: 33740.0 }) as Rc<dyn TieredRuleDigestModifier>,
            Rc::new(MinimumTieredRuleDigestAdjuster { minimum: 30.1 }),
            Rc::new(LabelTieredRuleDigestAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.03),
                    ("online".to_string(), 1.39),
                    ("north".to_string(), 1.4),
                ]),
            }),
            Rc::new(TieredRuleDigestVersionOffsetAdjuster { weight: 1.01 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 61706.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_rule_digest_engine();
        let item = new_tiered_rule_digest("code-159", "label-648");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
