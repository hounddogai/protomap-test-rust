use super::*;

use acme_kit::text;

/// Where a audit event batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventBatchStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AuditEventBatchStatus {
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

impl std::fmt::Display for AuditEventBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a audit event batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBatch {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub ceiling: f64,
    pub attempts: i64,
    pub discount: f64,
    pub checksum: String,
    pub name: String,
    pub summary: String,
    pub threshold: f64,
    pub starts_at: i64,
    pub status: AuditEventBatchStatus,
}

impl AuditEventBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventBatchStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventBatchStatus) -> bool {
        match self.status {
            AuditEventBatchStatus::Draft => {
                match next {
                    AuditEventBatchStatus::Active | AuditEventBatchStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventBatchStatus::Pending => {
                match next {
                    AuditEventBatchStatus::Active | AuditEventBatchStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventBatchStatus::Active => {
                match next {
                    AuditEventBatchStatus::Suspended | AuditEventBatchStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventBatchStatus::Suspended => {
                return next == AuditEventBatchStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event batch {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.threshold + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventBatch {
        AuditEventBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            ceiling: self.ceiling,
            attempts: self.attempts,
            discount: self.discount,
            checksum: self.checksum.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            threshold: self.threshold,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a audit event batch in status draft.
pub fn new_audit_event_batch(code: &str, title: &str) -> AuditEventBatch {
    AuditEventBatch {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        ceiling: 0.0,
        attempts: 0,
        discount: 0.0,
        checksum: "".to_string(),
        name: "".to_string(),
        summary: "".to_string(),
        threshold: 0.0,
        starts_at: 0,
        status: AuditEventBatchStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_audit_event_batch("code-272", "title-959");
        assert_eq!(item.status, AuditEventBatchStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_audit_event_batch("code-272", "title-959");
        if item.move_to(AuditEventBatchStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventBatchStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_audit_event_batch("code-272", "title-959");
        item.status = AuditEventBatchStatus::Closed;
        assert!(
            item.move_to(AuditEventBatchStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
