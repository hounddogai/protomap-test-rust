use super::*;

use acme_kit::text;

/// Where a regional audit event batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalAuditEventBatchStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalAuditEventBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalAuditEventBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional audit event batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventBatch {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub attempts: i64,
    pub burst: i64,
    pub region: String,
    pub timezone: String,
    pub status: RegionalAuditEventBatchStatus,
}

impl RegionalAuditEventBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalAuditEventBatchStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalAuditEventBatchStatus) -> bool {
        match self.status {
            RegionalAuditEventBatchStatus::Proposed => {
                match next {
                    RegionalAuditEventBatchStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalAuditEventBatchStatus::Approved => {
                match next {
                    RegionalAuditEventBatchStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalAuditEventBatchStatus::Rejected => {
                return next == RegionalAuditEventBatchStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalAuditEventBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional audit event batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional audit event batch", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.attempts as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalAuditEventBatch {
        RegionalAuditEventBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            attempts: self.attempts,
            burst: self.burst,
            region: self.region.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional audit event batch in status proposed.
pub fn new_regional_audit_event_batch(code: &str, label: &str) -> RegionalAuditEventBatch {
    RegionalAuditEventBatch {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        attempts: 0,
        burst: 0,
        region: "".to_string(),
        timezone: "".to_string(),
        status: RegionalAuditEventBatchStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_audit_event_batch("code-196", "label-32");
        assert_eq!(item.status, RegionalAuditEventBatchStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_regional_audit_event_batch("code-196", "label-32");
        if item.move_to(RegionalAuditEventBatchStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalAuditEventBatchStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_audit_event_batch("code-196", "label-32");
        item.status = RegionalAuditEventBatchStatus::Withdrawn;
        assert!(
            item.move_to(RegionalAuditEventBatchStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
