use super::*;

use acme_kit::text;

/// Where a lenient overage plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientOveragePlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientOveragePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LenientOveragePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient overage plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientOveragePlan {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub fingerprint: String,
    pub cipher: String,
    pub attempts: i64,
    pub expires_at: i64,
    pub rate: f64,
    pub sequence: i64,
    pub currency: String,
    pub status: LenientOveragePlanStatus,
}

impl LenientOveragePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientOveragePlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientOveragePlanStatus) -> bool {
        match self.status {
            LenientOveragePlanStatus::Open => {
                return next == LenientOveragePlanStatus::Resolved;
            }
            LenientOveragePlanStatus::InReview => {
                match next {
                    LenientOveragePlanStatus::Resolved | LenientOveragePlanStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientOveragePlanStatus::Resolved => {
                return next == LenientOveragePlanStatus::Reopened || next == LenientOveragePlanStatus::Archived;
            }
            LenientOveragePlanStatus::Reopened => {
                match next {
                    LenientOveragePlanStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientOveragePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient overage plan {} cannot move from {} to {}",
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
        format!("{} {}", "lenient overage plan", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.attempts as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientOveragePlan {
        LenientOveragePlan {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            fingerprint: self.fingerprint.clone(),
            cipher: self.cipher.clone(),
            attempts: self.attempts,
            expires_at: self.expires_at,
            rate: self.rate,
            sequence: self.sequence,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a lenient overage plan in status open.
pub fn new_lenient_overage_plan(code: &str, title: &str) -> LenientOveragePlan {
    LenientOveragePlan {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        fingerprint: "".to_string(),
        cipher: "".to_string(),
        attempts: 0,
        expires_at: 0,
        rate: 0.0,
        sequence: 0,
        currency: "".to_string(),
        status: LenientOveragePlanStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_overage_plan("code-173", "title-675");
        assert_eq!(item.status, LenientOveragePlanStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_lenient_overage_plan("code-173", "title-675");
        if item.move_to(LenientOveragePlanStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientOveragePlanStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_overage_plan("code-173", "title-675");
        item.status = LenientOveragePlanStatus::Archived;
        assert!(
            item.move_to(LenientOveragePlanStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
