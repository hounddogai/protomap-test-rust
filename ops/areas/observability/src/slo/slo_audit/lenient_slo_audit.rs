use super::*;

use acme_kit::text;

/// Where a lenient slo audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSloAuditStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientSloAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientSloAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient slo audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloAudit {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub key_id: String,
    pub summary: String,
    pub unit: String,
    pub description: String,
    pub host: String,
    pub updated_at: i64,
    pub status: LenientSloAuditStatus,
}

impl LenientSloAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSloAuditStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSloAuditStatus) -> bool {
        match self.status {
            LenientSloAuditStatus::Proposed => {
                return next == LenientSloAuditStatus::Approved || next == LenientSloAuditStatus::Withdrawn;
            }
            LenientSloAuditStatus::Approved => {
                match next {
                    LenientSloAuditStatus::Rejected | LenientSloAuditStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSloAuditStatus::Rejected => {
                return next == LenientSloAuditStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientSloAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient slo audit {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.updated_at as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSloAudit {
        LenientSloAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            key_id: self.key_id.clone(),
            summary: self.summary.clone(),
            unit: self.unit.clone(),
            description: self.description.clone(),
            host: self.host.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient slo audit in status proposed.
pub fn new_lenient_slo_audit(code: &str, label: &str) -> LenientSloAudit {
    LenientSloAudit {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        key_id: "".to_string(),
        summary: "".to_string(),
        unit: "".to_string(),
        description: "".to_string(),
        host: "".to_string(),
        updated_at: 0,
        status: LenientSloAuditStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_lenient_slo_audit("code-817", "label-410");
        assert_eq!(item.status, LenientSloAuditStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_lenient_slo_audit("code-817", "label-410");
        if item.move_to(LenientSloAuditStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSloAuditStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_lenient_slo_audit("code-817", "label-410");
        item.status = LenientSloAuditStatus::Withdrawn;
        assert!(
            item.move_to(LenientSloAuditStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
