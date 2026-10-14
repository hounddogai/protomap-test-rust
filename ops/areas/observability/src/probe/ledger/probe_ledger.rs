use super::*;

use acme_kit::text;

/// Where a probe ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ProbeLedgerStatus {
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

impl std::fmt::Display for ProbeLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One probe ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeLedger {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub ceiling: f64,
    pub starts_at: i64,
    pub fingerprint: String,
    pub issuer: String,
    pub retries: i64,
    pub rotation_days: i64,
    pub margin: f64,
    pub status: ProbeLedgerStatus,
}

impl ProbeLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeLedgerStatus) -> bool {
        match self.status {
            ProbeLedgerStatus::New => {
                match next {
                    ProbeLedgerStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeLedgerStatus::Queued => {
                return next == ProbeLedgerStatus::Failed;
            }
            ProbeLedgerStatus::Running => {
                match next {
                    ProbeLedgerStatus::Done | ProbeLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeLedgerStatus::Done => {
                match next {
                    ProbeLedgerStatus::Failed => {
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

    pub fn move_to(&mut self, next: ProbeLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe ledger {} cannot move from {} to {}",
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

    pub fn projected_exposure(&self) -> f64 {
        (self.margin * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeLedger {
        ProbeLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            ceiling: self.ceiling,
            starts_at: self.starts_at,
            fingerprint: self.fingerprint.clone(),
            issuer: self.issuer.clone(),
            retries: self.retries,
            rotation_days: self.rotation_days,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a probe ledger in status new.
pub fn new_probe_ledger(code: &str, title: &str) -> ProbeLedger {
    ProbeLedger {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        ceiling: 0.0,
        starts_at: 0,
        fingerprint: "".to_string(),
        issuer: "".to_string(),
        retries: 0,
        rotation_days: 0,
        margin: 0.0,
        status: ProbeLedgerStatus::New,
    }
}
