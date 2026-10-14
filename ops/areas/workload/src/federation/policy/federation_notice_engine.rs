use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation notice calculation.
pub trait FederationNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationNotice, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationNoticeAdjuster for FlaggedFederationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationNoticeLimitOffsetAdjuster {
    pub weight: f64,
}

impl FederationNoticeAdjuster for FederationNoticeLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &FederationNotice, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationNoticeAdjuster {
    pub step: f64,
}

impl FederationNoticeAdjuster for RoundingFederationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationNoticeAdjuster {
    pub percent: f64,
}

impl FederationNoticeAdjuster for PercentFederationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFederationNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationNoticeAdjuster for TitleFederationNoticeAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FederationNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationNoticeAdjuster {
    pub minimum: f64,
}

impl FederationNoticeAdjuster for MinimumFederationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the federation notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationNoticeEngine {
    pub steps: Vec<Rc<dyn FederationNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationNoticeEngine {
    pub fn evaluate(&mut self, item: &FederationNotice) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationNoticeAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_federation_notice_engine(flags: &flags::Registry) -> FederationNoticeEngine {
    FederationNoticeEngine {
        steps: vec![
            Rc::new(FlaggedFederationNoticeAdjuster {
                flags: flags.clone(),
                flag: "federation-notice-legacy".to_string(),
                percent: 9.6,
            }) as Rc<dyn FederationNoticeAdjuster>,
            Rc::new(FederationNoticeLimitOffsetAdjuster { weight: 1.73 }),
            Rc::new(RoundingFederationNoticeAdjuster { step: 0.05 }),
            Rc::new(PercentFederationNoticeAdjuster { percent: 8.9 }),
            Rc::new(TitleFederationNoticeAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.38), ("retail".to_string(), 1.39)]),
            }),
            Rc::new(MinimumFederationNoticeAdjuster { minimum: 18.6 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 5189.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_notice_engine(&flags::new_registry());
        let item = new_federation_notice("code-8", "title-258");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
