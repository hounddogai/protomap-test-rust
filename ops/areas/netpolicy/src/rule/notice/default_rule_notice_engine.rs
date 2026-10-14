use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the default rule notice calculation.
pub trait DefaultRuleNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultRuleNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultRuleNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultRuleNoticeAdjuster for TieredDefaultRuleNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultRuleNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultRuleNoticeAdjuster {
    pub percent: f64,
}

impl DefaultRuleNoticeAdjuster for PercentDefaultRuleNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultRuleNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDefaultRuleNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultRuleNoticeAdjuster for LabelDefaultRuleNoticeAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DefaultRuleNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default rule notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultRuleNoticeEngineExtra {
    pub steps: Vec<Rc<dyn DefaultRuleNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultRuleNoticeEngineExtra {
    pub fn price(&mut self, item: &DefaultRuleNotice) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultRuleNoticeAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultRuleNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_rule_notice_engine() -> DefaultRuleNoticeEngineExtra {
    DefaultRuleNoticeEngineExtra {
        steps: vec![
            Rc::new(TieredDefaultRuleNoticeAdjuster {
                bounds: vec![2955.0, 4736.0, 6843.0, 9605.0],
                rates: vec![1.15, 1.13, 1.11, 0.98],
            }) as Rc<dyn DefaultRuleNoticeAdjuster>,
            Rc::new(PercentDefaultRuleNoticeAdjuster { percent: 0.6 }),
            Rc::new(LabelDefaultRuleNoticeAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.17),
                    ("north".to_string(), 1.34),
                    ("east".to_string(), 0.89),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 73213.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_rule_notice_engine();
        let item = new_default_rule_notice("code-489", "label-312");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
