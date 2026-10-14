use super::*;

use acme_kit::text;

/// Where a legacy slo audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacySloAuditStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacySloAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacySloAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy slo audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySloAudit {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub rate: f64,
    pub attempts: i64,
    pub locale: String,
    pub notes: String,
    pub status: LegacySloAuditStatus,
}

impl LegacySloAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacySloAuditStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacySloAuditStatus) -> bool {
        match self.status {
            LegacySloAuditStatus::Proposed => {
                return next == LegacySloAuditStatus::Withdrawn;
            }
            LegacySloAuditStatus::Approved => {
                return next == LegacySloAuditStatus::Rejected || next == LegacySloAuditStatus::Withdrawn;
            }
            LegacySloAuditStatus::Rejected => {
                match next {
                    LegacySloAuditStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LegacySloAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy slo audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.attempts as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacySloAudit {
        LegacySloAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            rate: self.rate,
            attempts: self.attempts,
            locale: self.locale.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a legacy slo audit in status proposed.
pub fn new_legacy_slo_audit(code: &str, label: &str) -> LegacySloAudit {
    LegacySloAudit {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        rate: 0.0,
        attempts: 0,
        locale: "".to_string(),
        notes: "".to_string(),
        status: LegacySloAuditStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_legacy_slo_audit("code-454", "label-961");
        assert_eq!(item.status, LegacySloAuditStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_legacy_slo_audit("code-454", "label-961");
        if item.move_to(LegacySloAuditStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacySloAuditStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_legacy_slo_audit("code-454", "label-961");
        item.status = LegacySloAuditStatus::Withdrawn;
        assert!(
            item.move_to(LegacySloAuditStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
