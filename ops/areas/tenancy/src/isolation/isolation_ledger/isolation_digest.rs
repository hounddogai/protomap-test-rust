use super::*;

use acme_kit::text;

/// Where a isolation digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationDigestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IsolationDigestStatus {
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

impl std::fmt::Display for IsolationDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation digest as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDigest {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub sequence: i64,
    pub quantity: i64,
    pub retries: i64,
    pub namespace: String,
    pub ttl_seconds: i64,
    pub floor_value: f64,
    pub status: IsolationDigestStatus,
}

impl IsolationDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationDigestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationDigestStatus) -> bool {
        match self.status {
            IsolationDigestStatus::Planned => {
                match next {
                    IsolationDigestStatus::Scheduled | IsolationDigestStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationDigestStatus::Scheduled => {
                return next == IsolationDigestStatus::InProgress || next == IsolationDigestStatus::Completed;
            }
            IsolationDigestStatus::InProgress => {
                return next == IsolationDigestStatus::Completed || next == IsolationDigestStatus::Cancelled;
            }
            IsolationDigestStatus::Completed => {
                match next {
                    IsolationDigestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: IsolationDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.retries as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationDigest {
        IsolationDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            sequence: self.sequence,
            quantity: self.quantity,
            retries: self.retries,
            namespace: self.namespace.clone(),
            ttl_seconds: self.ttl_seconds,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a isolation digest in status planned.
pub fn new_isolation_digest(code: &str, title: &str) -> IsolationDigest {
    IsolationDigest {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        sequence: 0,
        quantity: 0,
        retries: 0,
        namespace: "".to_string(),
        ttl_seconds: 0,
        floor_value: 0.0,
        status: IsolationDigestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_isolation_digest("code-74", "title-48");
        assert_eq!(item.status, IsolationDigestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_isolation_digest("code-74", "title-48");
        if item.move_to(IsolationDigestStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationDigestStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_isolation_digest("code-74", "title-48");
        item.status = IsolationDigestStatus::Cancelled;
        assert!(
            item.move_to(IsolationDigestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
