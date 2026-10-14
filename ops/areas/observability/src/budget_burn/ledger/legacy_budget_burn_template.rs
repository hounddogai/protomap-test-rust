use super::*;

use acme_kit::text;

/// Where a legacy budget burn template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyBudgetBurnTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyBudgetBurnTemplateStatus {
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

impl std::fmt::Display for LegacyBudgetBurnTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy budget burn template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBudgetBurnTemplate {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub created_at: i64,
    pub cost: f64,
    pub maximum: f64,
    pub window_days: i64,
    pub timezone: String,
    pub discount: f64,
    pub visible: bool,
    pub status: LegacyBudgetBurnTemplateStatus,
}

impl LegacyBudgetBurnTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyBudgetBurnTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyBudgetBurnTemplateStatus) -> bool {
        match self.status {
            LegacyBudgetBurnTemplateStatus::Idle => {
                return next == LegacyBudgetBurnTemplateStatus::Ready
                    || next == LegacyBudgetBurnTemplateStatus::Draining;
            }
            LegacyBudgetBurnTemplateStatus::Warming => {
                return next == LegacyBudgetBurnTemplateStatus::Draining;
            }
            LegacyBudgetBurnTemplateStatus::Ready => {
                match next {
                    LegacyBudgetBurnTemplateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyBudgetBurnTemplateStatus::Draining => {
                return next == LegacyBudgetBurnTemplateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyBudgetBurnTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy budget burn template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.window_days as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyBudgetBurnTemplate {
        LegacyBudgetBurnTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            created_at: self.created_at,
            cost: self.cost,
            maximum: self.maximum,
            window_days: self.window_days,
            timezone: self.timezone.clone(),
            discount: self.discount,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy budget burn template in status idle.
pub fn new_legacy_budget_burn_template(code: &str, name: &str) -> LegacyBudgetBurnTemplate {
    LegacyBudgetBurnTemplate {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        created_at: 0,
        cost: 0.0,
        maximum: 0.0,
        window_days: 0,
        timezone: "".to_string(),
        discount: 0.0,
        visible: false,
        status: LegacyBudgetBurnTemplateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_budget_burn_template("code-917", "name-288");
        assert_eq!(item.status, LegacyBudgetBurnTemplateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_legacy_budget_burn_template("code-917", "name-288");
        if item.move_to(LegacyBudgetBurnTemplateStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyBudgetBurnTemplateStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_budget_burn_template("code-917", "name-288");
        item.status = LegacyBudgetBurnTemplateStatus::Stopped;
        assert!(
            item.move_to(LegacyBudgetBurnTemplateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
