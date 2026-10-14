use super::*;

use acme_kit::text;

/// Where a primary nonce score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryNonceScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryNonceScoreStatus {
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

impl std::fmt::Display for PrimaryNonceScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary nonce score as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryNonceScore {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub budget: f64,
    pub issuer: String,
    pub approved: bool,
    pub principal: String,
    pub unit: String,
    pub grace_days: i64,
    pub attempts: i64,
    pub status: PrimaryNonceScoreStatus,
}

impl PrimaryNonceScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryNonceScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryNonceScoreStatus) -> bool {
        match self.status {
            PrimaryNonceScoreStatus::New => {
                return next == PrimaryNonceScoreStatus::Running || next == PrimaryNonceScoreStatus::Failed;
            }
            PrimaryNonceScoreStatus::Queued => {
                return next == PrimaryNonceScoreStatus::Running || next == PrimaryNonceScoreStatus::Done;
            }
            PrimaryNonceScoreStatus::Running => {
                return next == PrimaryNonceScoreStatus::Done || next == PrimaryNonceScoreStatus::Failed;
            }
            PrimaryNonceScoreStatus::Done => {
                match next {
                    PrimaryNonceScoreStatus::Failed => {
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

    pub fn move_to(&mut self, next: PrimaryNonceScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary nonce score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary nonce score", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.attempts as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryNonceScore {
        PrimaryNonceScore {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            budget: self.budget,
            issuer: self.issuer.clone(),
            approved: self.approved,
            principal: self.principal.clone(),
            unit: self.unit.clone(),
            grace_days: self.grace_days,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary nonce score in status new.
pub fn new_primary_nonce_score(code: &str, label: &str) -> PrimaryNonceScore {
    PrimaryNonceScore {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        budget: 0.0,
        issuer: "".to_string(),
        approved: false,
        principal: "".to_string(),
        unit: "".to_string(),
        grace_days: 0,
        attempts: 0,
        status: PrimaryNonceScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_nonce_score("code-864", "label-290");
        assert_eq!(item.status, PrimaryNonceScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_primary_nonce_score("code-864", "label-290");
        if item.move_to(PrimaryNonceScoreStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryNonceScoreStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_nonce_score("code-864", "label-290");
        item.status = PrimaryNonceScoreStatus::Failed;
        assert!(
            item.move_to(PrimaryNonceScoreStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
