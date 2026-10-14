use super::*;

use acme_kit::text;

/// Where a strict overage agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictOverageAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictOverageAgreementStatus {
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

impl std::fmt::Display for StrictOverageAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict overage agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageAgreement {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub limit: i64,
    pub rate: f64,
    pub expires_at: i64,
    pub timezone: String,
    pub score: f64,
    pub discount: f64,
    pub priority: i64,
    pub status: StrictOverageAgreementStatus,
}

impl StrictOverageAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictOverageAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictOverageAgreementStatus) -> bool {
        match self.status {
            StrictOverageAgreementStatus::New => {
                return next == StrictOverageAgreementStatus::Running || next == StrictOverageAgreementStatus::Failed;
            }
            StrictOverageAgreementStatus::Queued => {
                match next {
                    StrictOverageAgreementStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictOverageAgreementStatus::Running => {
                match next {
                    StrictOverageAgreementStatus::Done | StrictOverageAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictOverageAgreementStatus::Done => {
                match next {
                    StrictOverageAgreementStatus::Failed => {
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

    pub fn move_to(&mut self, next: StrictOverageAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict overage agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict overage agreement", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.discount - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictOverageAgreement {
        StrictOverageAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            limit: self.limit,
            rate: self.rate,
            expires_at: self.expires_at,
            timezone: self.timezone.clone(),
            score: self.score,
            discount: self.discount,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict overage agreement in status new.
pub fn new_strict_overage_agreement(code: &str, name: &str) -> StrictOverageAgreement {
    StrictOverageAgreement {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        limit: 0,
        rate: 0.0,
        expires_at: 0,
        timezone: "".to_string(),
        score: 0.0,
        discount: 0.0,
        priority: 0,
        status: StrictOverageAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_overage_agreement("code-310", "name-595");
        assert_eq!(item.status, StrictOverageAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_strict_overage_agreement("code-310", "name-595");
        if item.move_to(StrictOverageAgreementStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictOverageAgreementStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_overage_agreement("code-310", "name-595");
        item.status = StrictOverageAgreementStatus::Failed;
        assert!(
            item.move_to(StrictOverageAgreementStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
