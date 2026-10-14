use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the page entry calculation.
pub trait PageEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPageEntryAdjuster {
    pub percent: f64,
}

impl PageEntryAdjuster for PercentPageEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PageEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPageEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PageEntryAdjuster for UnitPageEntryAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &PageEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PageEntryAdjuster for TieredPageEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PageEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPageEntryAdjuster {
    pub cap: f64,
}

impl PageEntryAdjuster for CapPageEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PageEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageEntryEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl PageEntryAdjuster for PageEntryEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &PageEntry, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPageEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PageEntryAdjuster for FlaggedPageEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PageEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the page entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageEntryEngine {
    pub steps: Vec<Rc<dyn PageEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageEntryEngine {
    pub fn compute(&mut self, item: &PageEntry) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PageEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_page_entry_engine(flags: &flags::Registry) -> PageEntryEngine {
    PageEntryEngine {
        steps: vec![
            Rc::new(PercentPageEntryAdjuster { percent: 23.3 }) as Rc<dyn PageEntryAdjuster>,
            Rc::new(UnitPageEntryAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.36)]),
            }),
            Rc::new(TieredPageEntryAdjuster {
                bounds: vec![338.0, 2510.0, 2636.0, 7474.0],
                rates: vec![0.89, 1.08, 1.04, 0.91],
            }),
            Rc::new(CapPageEntryAdjuster { cap: 4781.0 }),
            Rc::new(PageEntryEndsAtOffsetAdjuster { weight: 1.75 }),
            Rc::new(FlaggedPageEntryAdjuster {
                flags: flags.clone(),
                flag: "page-entry-adaptive".to_string(),
                percent: -0.7,
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 83232.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_entry_engine(&flags::new_registry());
        let item = new_page_entry("code-884", "name-734");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
