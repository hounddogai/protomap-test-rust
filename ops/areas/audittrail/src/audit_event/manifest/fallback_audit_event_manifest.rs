use super::*;

use acme_kit::text;

/// Where a fallback audit event manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackAuditEventManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackAuditEventManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for FallbackAuditEventManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback audit event manifest as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventManifest {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub cluster: String,
    pub retries: i64,
    pub budget: f64,
    pub scope: String,
    pub ttl_seconds: i64,
    pub status: FallbackAuditEventManifestStatus,
}

impl FallbackAuditEventManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackAuditEventManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackAuditEventManifestStatus) -> bool {
        match self.status {
            FallbackAuditEventManifestStatus::Planned => {
                match next {
                    FallbackAuditEventManifestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackAuditEventManifestStatus::Scheduled => {
                return next == FallbackAuditEventManifestStatus::InProgress
                    || next == FallbackAuditEventManifestStatus::Cancelled;
            }
            FallbackAuditEventManifestStatus::InProgress => {
                return next == FallbackAuditEventManifestStatus::Completed
                    || next == FallbackAuditEventManifestStatus::Cancelled;
            }
            FallbackAuditEventManifestStatus::Completed => {
                match next {
                    FallbackAuditEventManifestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FallbackAuditEventManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback audit event manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback audit event manifest", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.budget * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackAuditEventManifest {
        FallbackAuditEventManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            cluster: self.cluster.clone(),
            retries: self.retries,
            budget: self.budget,
            scope: self.scope.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a fallback audit event manifest in status planned.
pub fn new_fallback_audit_event_manifest(code: &str, title: &str) -> FallbackAuditEventManifest {
    FallbackAuditEventManifest {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        cluster: "".to_string(),
        retries: 0,
        budget: 0.0,
        scope: "".to_string(),
        ttl_seconds: 0,
        status: FallbackAuditEventManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_audit_event_manifest("code-672", "title-623");
        assert_eq!(item.status, FallbackAuditEventManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_fallback_audit_event_manifest("code-672", "title-623");
        if item.move_to(FallbackAuditEventManifestStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackAuditEventManifestStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_audit_event_manifest("code-672", "title-623");
        item.status = FallbackAuditEventManifestStatus::Cancelled;
        assert!(
            item.move_to(FallbackAuditEventManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
