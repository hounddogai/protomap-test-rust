use super::*;

use acme_kit::text;

/// Where a manual redaction target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRedactionTargetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualRedactionTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualRedactionTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual redaction target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionTarget {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub limit: i64,
    pub audience: String,
    pub locale: String,
    pub minimum: f64,
    pub created_at: i64,
    pub unit: String,
    pub phone: String,
    pub status: ManualRedactionTargetStatus,
}

impl ManualRedactionTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRedactionTargetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRedactionTargetStatus) -> bool {
        match self.status {
            ManualRedactionTargetStatus::Proposed => {
                return next == ManualRedactionTargetStatus::Rejected || next == ManualRedactionTargetStatus::Withdrawn;
            }
            ManualRedactionTargetStatus::Approved => {
                match next {
                    ManualRedactionTargetStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRedactionTargetStatus::Rejected => {
                return next == ManualRedactionTargetStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualRedactionTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual redaction target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.limit as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRedactionTarget {
        ManualRedactionTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            limit: self.limit,
            audience: self.audience.clone(),
            locale: self.locale.clone(),
            minimum: self.minimum,
            created_at: self.created_at,
            unit: self.unit.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a manual redaction target in status proposed.
pub fn new_manual_redaction_target(code: &str, label: &str) -> ManualRedactionTarget {
    ManualRedactionTarget {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        limit: 0,
        audience: "".to_string(),
        locale: "".to_string(),
        minimum: 0.0,
        created_at: 0,
        unit: "".to_string(),
        phone: "".to_string(),
        status: ManualRedactionTargetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_redaction_target("code-393", "label-102");
        assert_eq!(item.status, ManualRedactionTargetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_manual_redaction_target("code-393", "label-102");
        if item.move_to(ManualRedactionTargetStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRedactionTargetStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_redaction_target("code-393", "label-102");
        item.status = ManualRedactionTargetStatus::Withdrawn;
        assert!(
            item.move_to(ManualRedactionTargetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
