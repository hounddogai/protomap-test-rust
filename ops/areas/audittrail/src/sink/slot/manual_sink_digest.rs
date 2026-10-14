use super::*;

use acme_kit::text;

/// Where a manual sink digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualSinkDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualSinkDigestStatus {
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

impl std::fmt::Display for ManualSinkDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual sink digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkDigest {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub phone: String,
    pub grace_days: i64,
    pub cluster: String,
    pub archived: bool,
    pub created_at: i64,
    pub fingerprint: String,
    pub quantity: i64,
    pub status: ManualSinkDigestStatus,
}

impl ManualSinkDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualSinkDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualSinkDigestStatus) -> bool {
        match self.status {
            ManualSinkDigestStatus::New => {
                match next {
                    ManualSinkDigestStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSinkDigestStatus::Queued => {
                return next == ManualSinkDigestStatus::Running;
            }
            ManualSinkDigestStatus::Running => {
                return next == ManualSinkDigestStatus::Done || next == ManualSinkDigestStatus::Failed;
            }
            ManualSinkDigestStatus::Done => {
                return next == ManualSinkDigestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualSinkDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual sink digest {} cannot move from {} to {}",
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

    pub fn net_exposure(&self) -> f64 {
        ((self.grace_days as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualSinkDigest {
        ManualSinkDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            phone: self.phone.clone(),
            grace_days: self.grace_days,
            cluster: self.cluster.clone(),
            archived: self.archived,
            created_at: self.created_at,
            fingerprint: self.fingerprint.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a manual sink digest in status new.
pub fn new_manual_sink_digest(code: &str, label: &str) -> ManualSinkDigest {
    ManualSinkDigest {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        phone: "".to_string(),
        grace_days: 0,
        cluster: "".to_string(),
        archived: false,
        created_at: 0,
        fingerprint: "".to_string(),
        quantity: 0,
        status: ManualSinkDigestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_sink_digest("code-862", "label-902");
        assert_eq!(item.status, ManualSinkDigestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_manual_sink_digest("code-862", "label-902");
        if item.move_to(ManualSinkDigestStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualSinkDigestStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_sink_digest("code-862", "label-902");
        item.status = ManualSinkDigestStatus::Failed;
        assert!(
            item.move_to(ManualSinkDigestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
