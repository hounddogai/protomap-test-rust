use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the regional retention notice calculation.
pub trait RegionalRetentionNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalRetentionNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRegionalRetentionNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalRetentionNoticeStep for LocaleRegionalRetentionNoticeAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RegionalRetentionNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalRetentionNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalRetentionNoticeStep for TieredRegionalRetentionNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalRetentionNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalRetentionNoticeAdjuster {
    pub minimum: f64,
}

impl RegionalRetentionNoticeStep for MinimumRegionalRetentionNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalRetentionNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionNoticeRevisionOffsetAdjuster {
    pub weight: f64,
}

impl RegionalRetentionNoticeStep for RegionalRetentionNoticeRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalRetentionNotice, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

/// Runs the regional retention notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalRetentionNoticeEngine {
    pub steps: Vec<Rc<dyn RegionalRetentionNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalRetentionNoticeEngine {
    pub fn price(&mut self, item: &RegionalRetentionNotice) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalRetentionNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalRetentionNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_retention_notice_engine() -> RegionalRetentionNoticeEngine {
    RegionalRetentionNoticeEngine {
        steps: vec![
            Rc::new(LocaleRegionalRetentionNoticeAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.78)]),
            }) as Rc<dyn RegionalRetentionNoticeStep>,
            Rc::new(TieredRegionalRetentionNoticeAdjuster {
                bounds: vec![3235.0, 3468.0],
                rates: vec![0.9, 1.05],
            }),
            Rc::new(MinimumRegionalRetentionNoticeAdjuster { minimum: 30.8 }),
            Rc::new(RegionalRetentionNoticeRevisionOffsetAdjuster { weight: 1.99 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 84389.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_retention_notice_engine();
        let item = new_regional_retention_notice("code-236", "title-203");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
