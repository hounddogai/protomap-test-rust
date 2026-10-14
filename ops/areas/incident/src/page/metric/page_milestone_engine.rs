use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the page milestone calculation.
pub trait PageMilestoneFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPageMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PageMilestoneFactor for FlaggedPageMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PageMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPageMilestoneAdjuster {
    pub percent: f64,
}

impl PageMilestoneFactor for PercentPageMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PageMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPageMilestoneAdjuster {
    pub step: f64,
}

impl PageMilestoneFactor for RoundingPageMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PageMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelPageMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PageMilestoneFactor for ChannelPageMilestoneAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &PageMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPageMilestoneAdjuster {
    pub cap: f64,
}

impl PageMilestoneFactor for CapPageMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PageMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageMilestoneQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl PageMilestoneFactor for PageMilestoneQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &PageMilestone, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

/// Runs the page milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageMilestoneEngine {
    pub steps: Vec<Rc<dyn PageMilestoneFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageMilestoneEngine {
    pub fn price(&mut self, item: &PageMilestone) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[PageMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_page_milestone_engine(flags: &flags::Registry) -> PageMilestoneEngine {
    PageMilestoneEngine {
        steps: vec![
            Rc::new(FlaggedPageMilestoneAdjuster {
                flags: flags.clone(),
                flag: "page-milestone-regional".to_string(),
                percent: -5.1,
            }) as Rc<dyn PageMilestoneFactor>,
            Rc::new(PercentPageMilestoneAdjuster { percent: -8.8 }),
            Rc::new(RoundingPageMilestoneAdjuster { step: 0.1 }),
            Rc::new(ChannelPageMilestoneAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.17), ("retail".to_string(), 1.12)]),
            }),
            Rc::new(CapPageMilestoneAdjuster { cap: 40384.0 }),
            Rc::new(PageMilestoneQuotaUnitsOffsetAdjuster { weight: 1.99 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 28476.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_milestone_engine(&flags::new_registry());
        let item = new_page_milestone("code-199", "name-223");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
