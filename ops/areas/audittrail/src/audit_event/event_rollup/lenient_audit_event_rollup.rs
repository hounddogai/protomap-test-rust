use super::*;

use acme_kit::text;

/// Where a lenient audit event rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientAuditEventRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientAuditEventRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientAuditEventRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient audit event rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientAuditEventRollup {
    pub code: String,
    pub name: String,
    pub title: String,
    pub country: String,
    pub approved: bool,
    pub score: f64,
    pub ceiling: f64,
    pub checksum: String,
    pub floor_value: f64,
    pub namespace: String,
    pub status: LenientAuditEventRollupStatus,
}

impl LenientAuditEventRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientAuditEventRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientAuditEventRollupStatus) -> bool {
        match self.status {
            LenientAuditEventRollupStatus::Created => {
                match next {
                    LenientAuditEventRollupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAuditEventRollupStatus::Validated => {
                match next {
                    LenientAuditEventRollupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAuditEventRollupStatus::Published => {
                match next {
                    LenientAuditEventRollupStatus::Retired => {
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

    pub fn move_to(&mut self, next: LenientAuditEventRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient audit event rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.floor_value * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientAuditEventRollup {
        LenientAuditEventRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            approved: self.approved,
            score: self.score,
            ceiling: self.ceiling,
            checksum: self.checksum.clone(),
            floor_value: self.floor_value,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a lenient audit event rollup in status created.
pub fn new_lenient_audit_event_rollup(code: &str, name: &str) -> LenientAuditEventRollup {
    LenientAuditEventRollup {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        country: "".to_string(),
        approved: false,
        score: 0.0,
        ceiling: 0.0,
        checksum: "".to_string(),
        floor_value: 0.0,
        namespace: "".to_string(),
        status: LenientAuditEventRollupStatus::Created,
    }
}
