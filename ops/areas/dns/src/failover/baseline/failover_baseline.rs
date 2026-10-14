use super::*;

use acme_kit::text;

/// Where a failover baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverBaselineStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FailoverBaselineStatus {
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

impl std::fmt::Display for FailoverBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a failover baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBaseline {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub ratio: f64,
    pub cluster: String,
    pub starts_at: i64,
    pub approved: bool,
    pub margin: f64,
    pub expires_at: i64,
    pub status: FailoverBaselineStatus,
}

impl FailoverBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverBaselineStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverBaselineStatus) -> bool {
        match self.status {
            FailoverBaselineStatus::New => {
                return next == FailoverBaselineStatus::Queued;
            }
            FailoverBaselineStatus::Queued => {
                return next == FailoverBaselineStatus::Running || next == FailoverBaselineStatus::Failed;
            }
            FailoverBaselineStatus::Running => {
                match next {
                    FailoverBaselineStatus::Done | FailoverBaselineStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverBaselineStatus::Done => {
                return next == FailoverBaselineStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover baseline {} cannot move from {} to {}",
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
        format!("{} {}", "failover baseline", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.expires_at as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverBaseline {
        FailoverBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            ratio: self.ratio,
            cluster: self.cluster.clone(),
            starts_at: self.starts_at,
            approved: self.approved,
            margin: self.margin,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a failover baseline in status new.
pub fn new_failover_baseline(code: &str, name: &str) -> FailoverBaseline {
    FailoverBaseline {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        ratio: 0.0,
        cluster: "".to_string(),
        starts_at: 0,
        approved: false,
        margin: 0.0,
        expires_at: 0,
        status: FailoverBaselineStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_failover_baseline("code-918", "name-17");
        assert_eq!(item.status, FailoverBaselineStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_failover_baseline("code-918", "name-17");
        if item.move_to(FailoverBaselineStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverBaselineStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_failover_baseline("code-918", "name-17");
        item.status = FailoverBaselineStatus::Failed;
        assert!(
            item.move_to(FailoverBaselineStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
