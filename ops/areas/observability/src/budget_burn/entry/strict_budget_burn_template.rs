use super::*;

use acme_kit::text;

/// Where a strict budget burn template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictBudgetBurnTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictBudgetBurnTemplateStatus {
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

impl std::fmt::Display for StrictBudgetBurnTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict budget burn template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnTemplate {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub locale: String,
    pub tenant_id: String,
    pub created_at: i64,
    pub country: String,
    pub phone: String,
    pub amount: f64,
    pub status: StrictBudgetBurnTemplateStatus,
}

impl StrictBudgetBurnTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictBudgetBurnTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictBudgetBurnTemplateStatus) -> bool {
        match self.status {
            StrictBudgetBurnTemplateStatus::Idle => {
                match next {
                    StrictBudgetBurnTemplateStatus::Ready | StrictBudgetBurnTemplateStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictBudgetBurnTemplateStatus::Warming => {
                return next == StrictBudgetBurnTemplateStatus::Stopped;
            }
            StrictBudgetBurnTemplateStatus::Ready => {
                match next {
                    StrictBudgetBurnTemplateStatus::Draining | StrictBudgetBurnTemplateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictBudgetBurnTemplateStatus::Draining => {
                return next == StrictBudgetBurnTemplateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictBudgetBurnTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict budget burn template {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.created_at as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictBudgetBurnTemplate {
        StrictBudgetBurnTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            locale: self.locale.clone(),
            tenant_id: self.tenant_id.clone(),
            created_at: self.created_at,
            country: self.country.clone(),
            phone: self.phone.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a strict budget burn template in status idle.
pub fn new_strict_budget_burn_template(code: &str, label: &str) -> StrictBudgetBurnTemplate {
    StrictBudgetBurnTemplate {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        locale: "".to_string(),
        tenant_id: "".to_string(),
        created_at: 0,
        country: "".to_string(),
        phone: "".to_string(),
        amount: 0.0,
        status: StrictBudgetBurnTemplateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_budget_burn_template("code-977", "label-929");
        assert_eq!(item.status, StrictBudgetBurnTemplateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_strict_budget_burn_template("code-977", "label-929");
        if item.move_to(StrictBudgetBurnTemplateStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictBudgetBurnTemplateStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_budget_burn_template("code-977", "label-929");
        item.status = StrictBudgetBurnTemplateStatus::Stopped;
        assert!(
            item.move_to(StrictBudgetBurnTemplateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
