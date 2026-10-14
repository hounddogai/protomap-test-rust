use super::*;

use acme_kit::text;

/// Where a credential score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CredentialScoreStatus {
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

impl std::fmt::Display for CredentialScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One credential score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialScore {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub region: String,
    pub channel: String,
    pub country: String,
    pub summary: String,
    pub source: String,
    pub pinned: bool,
    pub quantity: i64,
    pub budget: f64,
    pub status: CredentialScoreStatus,
}

impl CredentialScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialScoreStatus) -> bool {
        match self.status {
            CredentialScoreStatus::Open => {
                match next {
                    CredentialScoreStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialScoreStatus::InReview => {
                match next {
                    CredentialScoreStatus::Reopened | CredentialScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialScoreStatus::Resolved => {
                match next {
                    CredentialScoreStatus::Reopened | CredentialScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialScoreStatus::Reopened => {
                return next == CredentialScoreStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "credential score", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.budget + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialScore {
        CredentialScore {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            region: self.region.clone(),
            channel: self.channel.clone(),
            country: self.country.clone(),
            summary: self.summary.clone(),
            source: self.source.clone(),
            pinned: self.pinned,
            quantity: self.quantity,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a credential score in status open.
pub fn new_credential_score(code: &str, label: &str) -> CredentialScore {
    CredentialScore {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        region: "".to_string(),
        channel: "".to_string(),
        country: "".to_string(),
        summary: "".to_string(),
        source: "".to_string(),
        pinned: false,
        quantity: 0,
        budget: 0.0,
        status: CredentialScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_credential_score("code-369", "label-937");
        assert_eq!(item.status, CredentialScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_credential_score("code-369", "label-937");
        if item.move_to(CredentialScoreStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialScoreStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_credential_score("code-369", "label-937");
        item.status = CredentialScoreStatus::Archived;
        assert!(
            item.move_to(CredentialScoreStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
