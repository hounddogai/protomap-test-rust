use super::*;

use acme_kit::text;

/// Where a regional token batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalTokenBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalTokenBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalTokenBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional token batch as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTokenBatch {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub budget: f64,
    pub contact_email: String,
    pub grace_days: i64,
    pub status: RegionalTokenBatchStatus,
}

impl RegionalTokenBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalTokenBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalTokenBatchStatus) -> bool {
        match self.status {
            RegionalTokenBatchStatus::Requested => {
                match next {
                    RegionalTokenBatchStatus::Granted | RegionalTokenBatchStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalTokenBatchStatus::Granted => {
                match next {
                    RegionalTokenBatchStatus::Revoked | RegionalTokenBatchStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalTokenBatchStatus::Revoked => {
                match next {
                    RegionalTokenBatchStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionalTokenBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional token batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional token batch", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.grace_days as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalTokenBatch {
        RegionalTokenBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            budget: self.budget,
            contact_email: self.contact_email.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional token batch in status requested.
pub fn new_regional_token_batch(code: &str, title: &str) -> RegionalTokenBatch {
    RegionalTokenBatch {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        budget: 0.0,
        contact_email: "".to_string(),
        grace_days: 0,
        status: RegionalTokenBatchStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_regional_token_batch("code-262", "title-477");
        assert_eq!(item.status, RegionalTokenBatchStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_regional_token_batch("code-262", "title-477");
        if item.move_to(RegionalTokenBatchStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalTokenBatchStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_regional_token_batch("code-262", "title-477");
        item.status = RegionalTokenBatchStatus::Expired;
        assert!(
            item.move_to(RegionalTokenBatchStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
