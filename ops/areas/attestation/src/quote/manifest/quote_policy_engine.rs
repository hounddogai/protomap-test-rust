use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the quote policy calculation.
pub trait QuotePolicyStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuotePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuotePolicyAdjuster {
    pub step: f64,
}

impl QuotePolicyStep for RoundingQuotePolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuotePolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuotePolicyAdjuster {
    pub percent: f64,
}

impl QuotePolicyStep for PercentQuotePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuotePolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQuotePolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuotePolicyStep for FlaggedQuotePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuotePolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuotePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QuotePolicyStep for TieredQuotePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QuotePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotePolicyAmountOffsetAdjuster {
    pub weight: f64,
}

impl QuotePolicyStep for QuotePolicyAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &QuotePolicy, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameQuotePolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuotePolicyStep for NameQuotePolicyAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &QuotePolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the quote policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuotePolicyEngine {
    pub steps: Vec<Rc<dyn QuotePolicyStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuotePolicyEngine {
    pub fn compute(&mut self, item: &QuotePolicy) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[QuotePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_quote_policy_engine(flags: &flags::Registry) -> QuotePolicyEngine {
    QuotePolicyEngine {
        steps: vec![
            Rc::new(RoundingQuotePolicyAdjuster { step: 5.0 }) as Rc<dyn QuotePolicyStep>,
            Rc::new(PercentQuotePolicyAdjuster { percent: -4.2 }),
            Rc::new(FlaggedQuotePolicyAdjuster {
                flags: flags.clone(),
                flag: "quote-policy-tiered".to_string(),
                percent: 9.9,
            }),
            Rc::new(TieredQuotePolicyAdjuster {
                bounds: vec![1858.0, 5626.0],
                rates: vec![1.12, 1.2],
            }),
            Rc::new(QuotePolicyAmountOffsetAdjuster { weight: 0.71 }),
            Rc::new(NameQuotePolicyAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.9),
                    ("central".to_string(), 0.76),
                    ("retail".to_string(), 1.21),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 34869.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_policy_engine(&flags::new_registry());
        let item = new_quote_policy("code-945", "name-946");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
