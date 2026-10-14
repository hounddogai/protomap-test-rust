use super::*;

use acme_kit::text;

/// Where a policy digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PolicyDigestStatus {
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

impl std::fmt::Display for PolicyDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDigest {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub limit: i64,
    pub attempts: i64,
    pub minimum: f64,
    pub ceiling: f64,
    pub active: bool,
    pub reviewer: String,
    pub created_at: i64,
    pub duration_minutes: i64,
    pub status: PolicyDigestStatus,
}

impl PolicyDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyDigestStatus) -> bool {
        match self.status {
            PolicyDigestStatus::New => {
                return next == PolicyDigestStatus::Running;
            }
            PolicyDigestStatus::Queued => {
                return next == PolicyDigestStatus::Running || next == PolicyDigestStatus::Done;
            }
            PolicyDigestStatus::Running => {
                match next {
                    PolicyDigestStatus::Done | PolicyDigestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyDigestStatus::Done => {
                return next == PolicyDigestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.created_at as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyDigest {
        PolicyDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            limit: self.limit,
            attempts: self.attempts,
            minimum: self.minimum,
            ceiling: self.ceiling,
            active: self.active,
            reviewer: self.reviewer.clone(),
            created_at: self.created_at,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a policy digest in status new.
pub fn new_policy_digest(code: &str, name: &str) -> PolicyDigest {
    PolicyDigest {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        limit: 0,
        attempts: 0,
        minimum: 0.0,
        ceiling: 0.0,
        active: false,
        reviewer: "".to_string(),
        created_at: 0,
        duration_minutes: 0,
        status: PolicyDigestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_policy_digest("code-356", "name-31");
        assert_eq!(item.status, PolicyDigestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_policy_digest("code-356", "name-31");
        if item.move_to(PolicyDigestStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyDigestStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_policy_digest("code-356", "name-31");
        item.status = PolicyDigestStatus::Failed;
        assert!(item.move_to(PolicyDigestStatus::New).is_err(), "expected move to fail");
    }
}
