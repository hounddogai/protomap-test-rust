use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the page draft calculation.
pub trait PageDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageDraft, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPageDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PageDraftModifier for FlaggedPageDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PageDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageDraftAdjuster {
    pub minimum: f64,
}

impl PageDraftModifier for MinimumPageDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPageDraftAdjuster {
    pub cap: f64,
}

impl PageDraftModifier for CapPageDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PageDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageDraftUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl PageDraftModifier for PageDraftUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &PageDraft, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

/// Runs the page draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageDraftEngine {
    pub steps: Vec<Rc<dyn PageDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageDraftEngine {
    pub fn evaluate(&mut self, item: &PageDraft) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PageDraftModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PageDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_page_draft_engine(flags: &flags::Registry) -> PageDraftEngine {
    PageDraftEngine {
        steps: vec![
            Rc::new(FlaggedPageDraftAdjuster {
                flags: flags.clone(),
                flag: "page-draft-lenient".to_string(),
                percent: -3.2,
            }) as Rc<dyn PageDraftModifier>,
            Rc::new(MinimumPageDraftAdjuster { minimum: 12.8 }),
            Rc::new(CapPageDraftAdjuster { cap: 6207.0 }),
            Rc::new(PageDraftUpdatedAtOffsetAdjuster { weight: 0.83 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 6037.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_draft_engine(&flags::new_registry());
        let item = new_page_draft("code-472", "label-317");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
