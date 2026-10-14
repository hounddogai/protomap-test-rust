use super::*;

use acme_kit::text;

/// Where a policy preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyPreferenceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PolicyPreferenceStatus {
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

impl std::fmt::Display for PolicyPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a policy preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyPreference {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub starts_at: i64,
    pub host: String,
    pub archived: bool,
    pub batch_size: i64,
    pub fingerprint: String,
    pub quota_units: i64,
    pub status: PolicyPreferenceStatus,
}

impl PolicyPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyPreferenceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyPreferenceStatus) -> bool {
        match self.status {
            PolicyPreferenceStatus::New => {
                return next == PolicyPreferenceStatus::Done || next == PolicyPreferenceStatus::Failed;
            }
            PolicyPreferenceStatus::Queued => {
                return next == PolicyPreferenceStatus::Done;
            }
            PolicyPreferenceStatus::Running => {
                match next {
                    PolicyPreferenceStatus::Done | PolicyPreferenceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyPreferenceStatus::Done => {
                match next {
                    PolicyPreferenceStatus::Failed => {
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

    pub fn move_to(&mut self, next: PolicyPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy preference {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.batch_size as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyPreference {
        PolicyPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            starts_at: self.starts_at,
            host: self.host.clone(),
            archived: self.archived,
            batch_size: self.batch_size,
            fingerprint: self.fingerprint.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a policy preference in status new.
pub fn new_policy_preference(code: &str, name: &str) -> PolicyPreference {
    PolicyPreference {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        starts_at: 0,
        host: "".to_string(),
        archived: false,
        batch_size: 0,
        fingerprint: "".to_string(),
        quota_units: 0,
        status: PolicyPreferenceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_policy_preference("code-306", "name-544");
        assert_eq!(item.status, PolicyPreferenceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_policy_preference("code-306", "name-544");
        if item.move_to(PolicyPreferenceStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyPreferenceStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_policy_preference("code-306", "name-544");
        item.status = PolicyPreferenceStatus::Failed;
        assert!(
            item.move_to(PolicyPreferenceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
