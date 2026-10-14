use super::*;

use acme_kit::text;

/// Where a limit budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitBudgetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LimitBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LimitBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitBudget {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub score: f64,
    pub maximum: f64,
    pub reviewer: String,
    pub version: i64,
    pub pinned: bool,
    pub summary: String,
    pub status: LimitBudgetStatus,
}

impl LimitBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitBudgetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitBudgetStatus) -> bool {
        match self.status {
            LimitBudgetStatus::Requested => {
                return next == LimitBudgetStatus::Revoked;
            }
            LimitBudgetStatus::Granted => {
                match next {
                    LimitBudgetStatus::Revoked | LimitBudgetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitBudgetStatus::Revoked => {
                match next {
                    LimitBudgetStatus::Expired => {
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

    pub fn move_to(&mut self, next: LimitBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.maximum + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitBudget {
        LimitBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            score: self.score,
            maximum: self.maximum,
            reviewer: self.reviewer.clone(),
            version: self.version,
            pinned: self.pinned,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a limit budget in status requested.
pub fn new_limit_budget(code: &str, label: &str) -> LimitBudget {
    LimitBudget {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        score: 0.0,
        maximum: 0.0,
        reviewer: "".to_string(),
        version: 0,
        pinned: false,
        summary: "".to_string(),
        status: LimitBudgetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_limit_budget("code-375", "label-518");
        assert_eq!(item.status, LimitBudgetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_limit_budget("code-375", "label-518");
        if item.move_to(LimitBudgetStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitBudgetStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_limit_budget("code-375", "label-518");
        item.status = LimitBudgetStatus::Expired;
        assert!(
            item.move_to(LimitBudgetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
