use super::*;

use acme_kit::text;

/// Where a retention score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RetentionScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RetentionScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention score as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionScore {
    pub code: String,
    pub label: String,
    pub host: String,
    pub cluster: String,
    pub ratio: f64,
    pub owner_id: String,
    pub attempts: i64,
    pub status: RetentionScoreStatus,
}

impl RetentionScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionScoreStatus) -> bool {
        match self.status {
            RetentionScoreStatus::Requested => {
                return next == RetentionScoreStatus::Granted;
            }
            RetentionScoreStatus::Granted => {
                match next {
                    RetentionScoreStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionScoreStatus::Revoked => {
                match next {
                    RetentionScoreStatus::Expired => {
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

    pub fn move_to(&mut self, next: RetentionScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ratio - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionScore {
        RetentionScore {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            cluster: self.cluster.clone(),
            ratio: self.ratio,
            owner_id: self.owner_id.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a retention score in status requested.
pub fn new_retention_score(code: &str, label: &str) -> RetentionScore {
    RetentionScore {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        cluster: "".to_string(),
        ratio: 0.0,
        owner_id: "".to_string(),
        attempts: 0,
        status: RetentionScoreStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_retention_score("code-635", "label-556");
        assert_eq!(item.status, RetentionScoreStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_retention_score("code-635", "label-556");
        if item.move_to(RetentionScoreStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionScoreStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_retention_score("code-635", "label-556");
        item.status = RetentionScoreStatus::Expired;
        assert!(
            item.move_to(RetentionScoreStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
