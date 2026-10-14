use super::*;

use acme_kit::text;

/// Where a retention estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RetentionEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RetentionEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One retention estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEstimate {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub visible: bool,
    pub key_id: String,
    pub status: RetentionEstimateStatus,
}

impl RetentionEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionEstimateStatus) -> bool {
        match self.status {
            RetentionEstimateStatus::Requested => {
                match next {
                    RetentionEstimateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionEstimateStatus::Granted => {
                return next == RetentionEstimateStatus::Expired;
            }
            RetentionEstimateStatus::Revoked => {
                match next {
                    RetentionEstimateStatus::Expired => {
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

    pub fn move_to(&mut self, next: RetentionEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention estimate {} cannot move from {} to {}",
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
        format!("{} {}", "retention estimate", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> RetentionEstimate {
        RetentionEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            visible: self.visible,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a retention estimate in status requested.
pub fn new_retention_estimate(code: &str, label: &str) -> RetentionEstimate {
    RetentionEstimate {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        visible: false,
        key_id: "".to_string(),
        status: RetentionEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_retention_estimate("code-471", "label-220");
        assert_eq!(item.status, RetentionEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_retention_estimate("code-471", "label-220");
        if item.move_to(RetentionEstimateStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionEstimateStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_retention_estimate("code-471", "label-220");
        item.status = RetentionEstimateStatus::Expired;
        assert!(
            item.move_to(RetentionEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
