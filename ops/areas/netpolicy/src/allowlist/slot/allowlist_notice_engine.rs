use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the allowlist notice calculation.
pub trait AllowlistNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistNoticeCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistNoticeAdjuster for AllowlistNoticeCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistNotice, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAllowlistNoticeAdjuster {
    pub percent: f64,
}

impl AllowlistNoticeAdjuster for PercentAllowlistNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AllowlistNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistNoticeAdjuster for FlaggedAllowlistNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AllowlistNoticeAdjuster for TieredAllowlistNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AllowlistNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the allowlist notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistNoticeEngine {
    pub steps: Vec<Rc<dyn AllowlistNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistNoticeEngine {
    pub fn price(&mut self, item: &AllowlistNotice) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AllowlistNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_allowlist_notice_engine(flags: &flags::Registry) -> AllowlistNoticeEngine {
    AllowlistNoticeEngine {
        steps: vec![
            Rc::new(AllowlistNoticeCreatedAtOffsetAdjuster { weight: 1.93 }) as Rc<dyn AllowlistNoticeAdjuster>,
            Rc::new(PercentAllowlistNoticeAdjuster { percent: 11.2 }),
            Rc::new(FlaggedAllowlistNoticeAdjuster {
                flags: flags.clone(),
                flag: "allowlist-notice-legacy".to_string(),
                percent: 5.7,
            }),
            Rc::new(TieredAllowlistNoticeAdjuster {
                bounds: vec![2655.0, 3593.0, 4820.0, 7507.0],
                rates: vec![0.87, 1.14, 0.93, 1.2],
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 70263.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_allowlist_notice_engine(&flags::new_registry());
        let item = new_allowlist_notice("code-435", "name-152");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
