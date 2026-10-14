use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal budget burn template calculation.
pub trait SeasonalBudgetBurnTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalBudgetBurnTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalBudgetBurnTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalBudgetBurnTemplateModifier for TieredSeasonalBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalBudgetBurnTemplateAdjuster {
    pub step: f64,
}

impl SeasonalBudgetBurnTemplateModifier for RoundingSeasonalBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleSeasonalBudgetBurnTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalBudgetBurnTemplateModifier for TitleSeasonalBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &SeasonalBudgetBurnTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalBudgetBurnTemplateAdjuster {
    pub percent: f64,
}

impl SeasonalBudgetBurnTemplateModifier for PercentSeasonalBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalBudgetBurnTemplateAdjuster {
    pub minimum: f64,
}

impl SeasonalBudgetBurnTemplateModifier for MinimumSeasonalBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalBudgetBurnTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalBudgetBurnTemplateModifier for FlaggedSeasonalBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal budget burn template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalBudgetBurnTemplateEngine {
    pub steps: Vec<Rc<dyn SeasonalBudgetBurnTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalBudgetBurnTemplateEngine {
    pub fn run(&mut self, item: &SeasonalBudgetBurnTemplate) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalBudgetBurnTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_budget_burn_template_engine(flags: &flags::Registry) -> SeasonalBudgetBurnTemplateEngine {
    SeasonalBudgetBurnTemplateEngine {
        steps: vec![
            Rc::new(TieredSeasonalBudgetBurnTemplateAdjuster {
                bounds: vec![961.0, 1808.0, 3392.0, 4593.0],
                rates: vec![0.84, 0.9, 0.85, 0.98],
            }) as Rc<dyn SeasonalBudgetBurnTemplateModifier>,
            Rc::new(RoundingSeasonalBudgetBurnTemplateAdjuster { step: 0.25 }),
            Rc::new(TitleSeasonalBudgetBurnTemplateAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.08)]),
            }),
            Rc::new(PercentSeasonalBudgetBurnTemplateAdjuster { percent: 14.9 }),
            Rc::new(MinimumSeasonalBudgetBurnTemplateAdjuster { minimum: 21.7 }),
            Rc::new(FlaggedSeasonalBudgetBurnTemplateAdjuster {
                flags: flags.clone(),
                flag: "seasonal-budget-burn-template-weighted".to_string(),
                percent: 3.9,
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 69184.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_budget_burn_template_engine(&flags::new_registry());
        let item = new_seasonal_budget_burn_template("code-560", "title-237");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
