use super::*;

use acme_kit::text;

/// Where a rolling silence audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSilenceAuditStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingSilenceAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RollingSilenceAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling silence audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceAudit {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub contact_email: String,
    pub source: String,
    pub segment: String,
    pub capacity: i64,
    pub status: RollingSilenceAuditStatus,
}

impl RollingSilenceAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSilenceAuditStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSilenceAuditStatus) -> bool {
        match self.status {
            RollingSilenceAuditStatus::Draft => {
                match next {
                    RollingSilenceAuditStatus::Pending | RollingSilenceAuditStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSilenceAuditStatus::Pending => {
                return next == RollingSilenceAuditStatus::Suspended;
            }
            RollingSilenceAuditStatus::Active => {
                return next == RollingSilenceAuditStatus::Closed;
            }
            RollingSilenceAuditStatus::Suspended => {
                return next == RollingSilenceAuditStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingSilenceAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling silence audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> RollingSilenceAudit {
        RollingSilenceAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            contact_email: self.contact_email.clone(),
            source: self.source.clone(),
            segment: self.segment.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rolling silence audit in status draft.
pub fn new_rolling_silence_audit(code: &str, title: &str) -> RollingSilenceAudit {
    RollingSilenceAudit {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        contact_email: "".to_string(),
        source: "".to_string(),
        segment: "".to_string(),
        capacity: 0,
        status: RollingSilenceAuditStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_silence_audit("code-312", "title-303");
        assert_eq!(item.status, RollingSilenceAuditStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_rolling_silence_audit("code-312", "title-303");
        if item.move_to(RollingSilenceAuditStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingSilenceAuditStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_silence_audit("code-312", "title-303");
        item.status = RollingSilenceAuditStatus::Closed;
        assert!(
            item.move_to(RollingSilenceAuditStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
