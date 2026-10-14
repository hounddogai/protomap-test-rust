use super::*;

use acme_kit::text;

/// Where a usage score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl UsageScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for UsageScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a usage score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageScore {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub host: String,
    pub burst: i64,
    pub capacity: i64,
    pub status: UsageScoreStatus,
}

impl UsageScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageScoreStatus) -> bool {
        match self.status {
            UsageScoreStatus::Requested => {
                match next {
                    UsageScoreStatus::Granted | UsageScoreStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageScoreStatus::Granted => {
                match next {
                    UsageScoreStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageScoreStatus::Revoked => {
                match next {
                    UsageScoreStatus::Expired => {
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

    pub fn move_to(&mut self, next: UsageScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "usage score", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.capacity as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageScore {
        UsageScore {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            host: self.host.clone(),
            burst: self.burst,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a usage score in status requested.
pub fn new_usage_score(code: &str, label: &str) -> UsageScore {
    UsageScore {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        host: "".to_string(),
        burst: 0,
        capacity: 0,
        status: UsageScoreStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_usage_score("code-383", "label-121");
        assert_eq!(item.status, UsageScoreStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_usage_score("code-383", "label-121");
        if item.move_to(UsageScoreStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageScoreStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_usage_score("code-383", "label-121");
        item.status = UsageScoreStatus::Expired;
        assert!(
            item.move_to(UsageScoreStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
