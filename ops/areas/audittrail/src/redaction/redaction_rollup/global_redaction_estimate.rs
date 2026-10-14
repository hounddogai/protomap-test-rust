use super::*;

use acme_kit::text;

/// Where a global redaction estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRedactionEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalRedactionEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalRedactionEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global redaction estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionEstimate {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub capacity: i64,
    pub category: String,
    pub cipher: String,
    pub ratio: f64,
    pub status: GlobalRedactionEstimateStatus,
}

impl GlobalRedactionEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRedactionEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRedactionEstimateStatus) -> bool {
        match self.status {
            GlobalRedactionEstimateStatus::Requested => {
                match next {
                    GlobalRedactionEstimateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRedactionEstimateStatus::Granted => {
                return next == GlobalRedactionEstimateStatus::Revoked
                    || next == GlobalRedactionEstimateStatus::Expired;
            }
            GlobalRedactionEstimateStatus::Revoked => {
                match next {
                    GlobalRedactionEstimateStatus::Expired => {
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

    pub fn move_to(&mut self, next: GlobalRedactionEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global redaction estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global redaction estimate", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.burst as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalRedactionEstimate {
        GlobalRedactionEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            capacity: self.capacity,
            category: self.category.clone(),
            cipher: self.cipher.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a global redaction estimate in status requested.
pub fn new_global_redaction_estimate(code: &str, title: &str) -> GlobalRedactionEstimate {
    GlobalRedactionEstimate {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        capacity: 0,
        category: "".to_string(),
        cipher: "".to_string(),
        ratio: 0.0,
        status: GlobalRedactionEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_redaction_estimate("code-616", "title-754");
        assert_eq!(item.status, GlobalRedactionEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_global_redaction_estimate("code-616", "title-754");
        if item.move_to(GlobalRedactionEstimateStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalRedactionEstimateStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_redaction_estimate("code-616", "title-754");
        item.status = GlobalRedactionEstimateStatus::Expired;
        assert!(
            item.move_to(GlobalRedactionEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
