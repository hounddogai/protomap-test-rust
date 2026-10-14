use super::*;

use acme_kit::text;

/// Where a manual slo template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualSloTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualSloTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualSloTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual slo template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloTemplate {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub ends_at: i64,
    pub locale: String,
    pub limit: i64,
    pub status: ManualSloTemplateStatus,
}

impl ManualSloTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualSloTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualSloTemplateStatus) -> bool {
        match self.status {
            ManualSloTemplateStatus::Proposed => {
                return next == ManualSloTemplateStatus::Withdrawn;
            }
            ManualSloTemplateStatus::Approved => {
                return next == ManualSloTemplateStatus::Rejected || next == ManualSloTemplateStatus::Withdrawn;
            }
            ManualSloTemplateStatus::Rejected => {
                match next {
                    ManualSloTemplateStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualSloTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual slo template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual slo template", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.limit as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualSloTemplate {
        ManualSloTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            ends_at: self.ends_at,
            locale: self.locale.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a manual slo template in status proposed.
pub fn new_manual_slo_template(code: &str, label: &str) -> ManualSloTemplate {
    ManualSloTemplate {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        ends_at: 0,
        locale: "".to_string(),
        limit: 0,
        status: ManualSloTemplateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_slo_template("code-898", "label-833");
        assert_eq!(item.status, ManualSloTemplateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_manual_slo_template("code-898", "label-833");
        if item.move_to(ManualSloTemplateStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualSloTemplateStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_slo_template("code-898", "label-833");
        item.status = ManualSloTemplateStatus::Withdrawn;
        assert!(
            item.move_to(ManualSloTemplateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
