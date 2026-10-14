use super::*;

use acme_kit::text;

/// Where a audit event group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventGroupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AuditEventGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for AuditEventGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a audit event group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventGroup {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub issuer: String,
    pub retries: i64,
    pub threshold: f64,
    pub region: String,
    pub cipher: String,
    pub status: AuditEventGroupStatus,
}

impl AuditEventGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventGroupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventGroupStatus) -> bool {
        match self.status {
            AuditEventGroupStatus::New => {
                match next {
                    AuditEventGroupStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventGroupStatus::Queued => {
                return next == AuditEventGroupStatus::Running;
            }
            AuditEventGroupStatus::Running => {
                return next == AuditEventGroupStatus::Done || next == AuditEventGroupStatus::Failed;
            }
            AuditEventGroupStatus::Done => {
                match next {
                    AuditEventGroupStatus::Failed => {
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

    pub fn move_to(&mut self, next: AuditEventGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.threshold + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventGroup {
        AuditEventGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            issuer: self.issuer.clone(),
            retries: self.retries,
            threshold: self.threshold,
            region: self.region.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a audit event group in status new.
pub fn new_audit_event_group(code: &str, label: &str) -> AuditEventGroup {
    AuditEventGroup {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        issuer: "".to_string(),
        retries: 0,
        threshold: 0.0,
        region: "".to_string(),
        cipher: "".to_string(),
        status: AuditEventGroupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_audit_event_group("code-158", "label-870");
        assert_eq!(item.status, AuditEventGroupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_audit_event_group("code-158", "label-870");
        if item.move_to(AuditEventGroupStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventGroupStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_audit_event_group("code-158", "label-870");
        item.status = AuditEventGroupStatus::Failed;
        assert!(
            item.move_to(AuditEventGroupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
