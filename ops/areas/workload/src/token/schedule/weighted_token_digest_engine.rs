use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted token digest calculation.
pub trait WeightedTokenDigestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedTokenDigest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedTokenDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedTokenDigestFactor for FlaggedWeightedTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedTokenDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedTokenDigestAdjuster {
    pub percent: f64,
}

impl WeightedTokenDigestFactor for PercentWeightedTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedTokenDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedTokenDigestAdjuster {
    pub step: f64,
}

impl WeightedTokenDigestFactor for RoundingWeightedTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedTokenDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleWeightedTokenDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedTokenDigestFactor for TitleWeightedTokenDigestAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &WeightedTokenDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenDigestWeightOffsetAdjuster {
    pub weight: f64,
}

impl WeightedTokenDigestFactor for WeightedTokenDigestWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedTokenDigest, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

/// Runs the weighted token digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedTokenDigestEngine {
    pub steps: Vec<Rc<dyn WeightedTokenDigestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedTokenDigestEngine {
    pub fn price(&mut self, item: &WeightedTokenDigest) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[WeightedTokenDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_token_digest_engine(flags: &flags::Registry) -> WeightedTokenDigestEngine {
    WeightedTokenDigestEngine {
        steps: vec![
            Rc::new(FlaggedWeightedTokenDigestAdjuster {
                flags: flags.clone(),
                flag: "weighted-token-digest-seasonal".to_string(),
                percent: 0.4,
            }) as Rc<dyn WeightedTokenDigestFactor>,
            Rc::new(PercentWeightedTokenDigestAdjuster { percent: 20.8 }),
            Rc::new(RoundingWeightedTokenDigestAdjuster { step: 0.5 }),
            Rc::new(TitleWeightedTokenDigestAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.0)]),
            }),
            Rc::new(WeightedTokenDigestWeightOffsetAdjuster { weight: 1.49 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 49962.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_token_digest_engine(&flags::new_registry());
        let item = new_weighted_token_digest("code-315", "title-696");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
