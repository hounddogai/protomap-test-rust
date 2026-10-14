use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a seasonal budget burn template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalBudgetBurnTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalBudgetBurnTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for SeasonalBudgetBurnTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal budget burn template as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnTemplate {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub tenant_id: String,
    pub weight: f64,
    pub ratio: f64,
    pub summary: String,
    pub host: String,
    pub timezone: String,
    pub attributes: BTreeMap<String, String>,
    pub status: SeasonalBudgetBurnTemplateStatus,
}

impl SeasonalBudgetBurnTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalBudgetBurnTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalBudgetBurnTemplateStatus) -> bool {
        match self.status {
            SeasonalBudgetBurnTemplateStatus::Idle => {
                match next {
                    SeasonalBudgetBurnTemplateStatus::Draining | SeasonalBudgetBurnTemplateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalBudgetBurnTemplateStatus::Warming => {
                match next {
                    SeasonalBudgetBurnTemplateStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalBudgetBurnTemplateStatus::Ready => {
                match next {
                    SeasonalBudgetBurnTemplateStatus::Draining | SeasonalBudgetBurnTemplateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalBudgetBurnTemplateStatus::Draining => {
                return next == SeasonalBudgetBurnTemplateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalBudgetBurnTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal budget burn template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_load(&self) -> f64 {
        ((self.weight + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalBudgetBurnTemplate {
        SeasonalBudgetBurnTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            tenant_id: self.tenant_id.clone(),
            weight: self.weight,
            ratio: self.ratio,
            summary: self.summary.clone(),
            host: self.host.clone(),
            timezone: self.timezone.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal budget burn template in status idle.
pub fn new_seasonal_budget_burn_template(code: &str, title: &str) -> SeasonalBudgetBurnTemplate {
    SeasonalBudgetBurnTemplate {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        tenant_id: "".to_string(),
        weight: 0.0,
        ratio: 0.0,
        summary: "".to_string(),
        host: "".to_string(),
        timezone: "".to_string(),
        attributes: BTreeMap::new(),
        status: SeasonalBudgetBurnTemplateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_budget_burn_template("code-560", "title-237");
        assert_eq!(item.status, SeasonalBudgetBurnTemplateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_seasonal_budget_burn_template("code-560", "title-237");
        if item.move_to(SeasonalBudgetBurnTemplateStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalBudgetBurnTemplateStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_budget_burn_template("code-560", "title-237");
        item.status = SeasonalBudgetBurnTemplateStatus::Stopped;
        assert!(
            item.move_to(SeasonalBudgetBurnTemplateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
