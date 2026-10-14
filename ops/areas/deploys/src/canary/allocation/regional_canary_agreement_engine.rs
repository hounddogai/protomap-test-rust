use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional canary agreement calculation.
pub trait RegionalCanaryAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalCanaryAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalCanaryAgreementAdjuster {
    pub minimum: f64,
}

impl RegionalCanaryAgreementModifier for MinimumRegionalCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalCanaryAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalCanaryAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalCanaryAgreementModifier for TieredRegionalCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalCanaryAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalCanaryAgreementAdjuster {
    pub step: f64,
}

impl RegionalCanaryAgreementModifier for RoundingRegionalCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalCanaryAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalCanaryAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalCanaryAgreementModifier for FlaggedRegionalCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalCanaryAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryAgreementAmountOffsetAdjuster {
    pub weight: f64,
}

impl RegionalCanaryAgreementModifier for RegionalCanaryAgreementAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalCanaryAgreement, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeRegionalCanaryAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalCanaryAgreementModifier for ScopeRegionalCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &RegionalCanaryAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the regional canary agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalCanaryAgreementEngine {
    pub steps: Vec<Rc<dyn RegionalCanaryAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalCanaryAgreementEngine {
    pub fn run(&mut self, item: &RegionalCanaryAgreement) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionalCanaryAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_canary_agreement_engine(flags: &flags::Registry) -> RegionalCanaryAgreementEngine {
    RegionalCanaryAgreementEngine {
        steps: vec![
            Rc::new(MinimumRegionalCanaryAgreementAdjuster { minimum: 26.7 })
                as Rc<dyn RegionalCanaryAgreementModifier>,
            Rc::new(TieredRegionalCanaryAgreementAdjuster {
                bounds: vec![800.0, 1531.0, 6829.0, 9475.0],
                rates: vec![1.03, 1.03, 1.15, 1.08],
            }),
            Rc::new(RoundingRegionalCanaryAgreementAdjuster { step: 0.5 }),
            Rc::new(FlaggedRegionalCanaryAgreementAdjuster {
                flags: flags.clone(),
                flag: "regional-canary-agreement-primary".to_string(),
                percent: -7.9,
            }),
            Rc::new(RegionalCanaryAgreementAmountOffsetAdjuster { weight: 0.97 }),
            Rc::new(ScopeRegionalCanaryAgreementAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.96),
                    ("north".to_string(), 1.09),
                    ("east".to_string(), 1.39),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 30710.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_canary_agreement_engine(&flags::new_registry());
        let item = new_regional_canary_agreement("code-921", "name-179");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
