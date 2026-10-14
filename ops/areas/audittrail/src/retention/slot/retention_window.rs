use super::*;

use acme_kit::text;

/// Where a retention window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionWindowStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RetentionWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RetentionWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a retention window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionWindow {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub batch_size: i64,
    pub host: String,
    pub duration_minutes: i64,
    pub archived: bool,
    pub cluster: String,
    pub status: RetentionWindowStatus,
}

impl RetentionWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionWindowStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionWindowStatus) -> bool {
        match self.status {
            RetentionWindowStatus::Requested => {
                match next {
                    RetentionWindowStatus::Revoked | RetentionWindowStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionWindowStatus::Granted => {
                match next {
                    RetentionWindowStatus::Revoked | RetentionWindowStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionWindowStatus::Revoked => {
                match next {
                    RetentionWindowStatus::Expired => {
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

    pub fn move_to(&mut self, next: RetentionWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.minimum - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionWindow {
        RetentionWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            batch_size: self.batch_size,
            host: self.host.clone(),
            duration_minutes: self.duration_minutes,
            archived: self.archived,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a retention window in status requested.
pub fn new_retention_window(code: &str, name: &str) -> RetentionWindow {
    RetentionWindow {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        batch_size: 0,
        host: "".to_string(),
        duration_minutes: 0,
        archived: false,
        cluster: "".to_string(),
        status: RetentionWindowStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_retention_window("code-867", "name-557");
        assert_eq!(item.status, RetentionWindowStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_retention_window("code-867", "name-557");
        if item.move_to(RetentionWindowStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionWindowStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_retention_window("code-867", "name-557");
        item.status = RetentionWindowStatus::Expired;
        assert!(
            item.move_to(RetentionWindowStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
