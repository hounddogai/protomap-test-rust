use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy quote group calculation.
pub trait LegacyQuoteGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyQuoteGroup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyQuoteGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyQuoteGroupAdjuster for FlaggedLegacyQuoteGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyQuoteGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleLegacyQuoteGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyQuoteGroupAdjuster for LocaleLegacyQuoteGroupAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &LegacyQuoteGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyQuoteGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyQuoteGroupAdjuster for TieredLegacyQuoteGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyQuoteGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyQuoteGroupAdjuster {
    pub minimum: f64,
}

impl LegacyQuoteGroupAdjuster for MinimumLegacyQuoteGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyQuoteGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyQuoteGroupAdjuster {
    pub step: f64,
}

impl LegacyQuoteGroupAdjuster for RoundingLegacyQuoteGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyQuoteGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyQuoteGroupAdjuster {
    pub cap: f64,
}

impl LegacyQuoteGroupAdjuster for CapLegacyQuoteGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyQuoteGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the legacy quote group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyQuoteGroupEngine {
    pub steps: Vec<Rc<dyn LegacyQuoteGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyQuoteGroupEngine {
    pub fn price(&mut self, item: &LegacyQuoteGroup) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyQuoteGroupAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyQuoteGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_quote_group_engine(flags: &flags::Registry) -> LegacyQuoteGroupEngine {
    LegacyQuoteGroupEngine {
        steps: vec![
            Rc::new(FlaggedLegacyQuoteGroupAdjuster {
                flags: flags.clone(),
                flag: "legacy-quote-group-legacy".to_string(),
                percent: 3.9,
            }) as Rc<dyn LegacyQuoteGroupAdjuster>,
            Rc::new(LocaleLegacyQuoteGroupAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.27), ("central".to_string(), 1.31)]),
            }),
            Rc::new(TieredLegacyQuoteGroupAdjuster {
                bounds: vec![5994.0, 8427.0, 9152.0],
                rates: vec![0.84, 1.09, 1.19],
            }),
            Rc::new(MinimumLegacyQuoteGroupAdjuster { minimum: 2.2 }),
            Rc::new(RoundingLegacyQuoteGroupAdjuster { step: 0.5 }),
            Rc::new(CapLegacyQuoteGroupAdjuster { cap: 31490.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 60391.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_quote_group_engine(&flags::new_registry());
        let item = new_legacy_quote_group("code-211", "name-851");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
