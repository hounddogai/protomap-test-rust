use super::*;

use acme_kit::text;

/// Where a suspension threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionThresholdStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SuspensionThresholdStatus {
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

impl std::fmt::Display for SuspensionThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension threshold as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionThreshold {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub currency: String,
    pub issuer: String,
    pub retries: i64,
    pub cost: f64,
    pub status: SuspensionThresholdStatus,
}

impl SuspensionThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionThresholdStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionThresholdStatus) -> bool {
        match self.status {
            SuspensionThresholdStatus::Open => {
                match next {
                    SuspensionThresholdStatus::InReview | SuspensionThresholdStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionThresholdStatus::InReview => {
                return next == SuspensionThresholdStatus::Reopened;
            }
            SuspensionThresholdStatus::Resolved => {
                match next {
                    SuspensionThresholdStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionThresholdStatus::Reopened => {
                return next == SuspensionThresholdStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "suspension threshold", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.cost - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionThreshold {
        SuspensionThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            currency: self.currency.clone(),
            issuer: self.issuer.clone(),
            retries: self.retries,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a suspension threshold in status open.
pub fn new_suspension_threshold(code: &str, name: &str) -> SuspensionThreshold {
    SuspensionThreshold {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        currency: "".to_string(),
        issuer: "".to_string(),
        retries: 0,
        cost: 0.0,
        status: SuspensionThresholdStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_suspension_threshold("code-363", "name-656");
        assert_eq!(item.status, SuspensionThresholdStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_suspension_threshold("code-363", "name-656");
        if item.move_to(SuspensionThresholdStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionThresholdStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_suspension_threshold("code-363", "name-656");
        item.status = SuspensionThresholdStatus::Archived;
        assert!(
            item.move_to(SuspensionThresholdStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
