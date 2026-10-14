use super::*;

use acme_kit::text;

/// Where a manual verifier review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualVerifierReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualVerifierReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualVerifierReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual verifier review as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierReview {
    pub code: String,
    pub name: String,
    pub source: String,
    pub budget: f64,
    pub window_days: i64,
    pub country: String,
    pub title: String,
    pub principal: String,
    pub label: String,
    pub tags: Vec<String>,
    pub revision: i64,
    pub status: ManualVerifierReviewStatus,
}

impl ManualVerifierReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualVerifierReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualVerifierReviewStatus) -> bool {
        match self.status {
            ManualVerifierReviewStatus::Requested => {
                match next {
                    ManualVerifierReviewStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualVerifierReviewStatus::Granted => {
                match next {
                    ManualVerifierReviewStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualVerifierReviewStatus::Revoked => {
                return next == ManualVerifierReviewStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualVerifierReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual verifier review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.revision as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualVerifierReview {
        ManualVerifierReview {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            budget: self.budget,
            window_days: self.window_days,
            country: self.country.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a manual verifier review in status requested.
pub fn new_manual_verifier_review(code: &str, name: &str) -> ManualVerifierReview {
    ManualVerifierReview {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        budget: 0.0,
        window_days: 0,
        country: "".to_string(),
        title: "".to_string(),
        principal: "".to_string(),
        label: "".to_string(),
        tags: Vec::new(),
        revision: 0,
        status: ManualVerifierReviewStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_verifier_review("code-225", "name-993");
        assert_eq!(item.status, ManualVerifierReviewStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_manual_verifier_review("code-225", "name-993");
        if item.move_to(ManualVerifierReviewStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualVerifierReviewStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_verifier_review("code-225", "name-993");
        item.status = ManualVerifierReviewStatus::Expired;
        assert!(
            item.move_to(ManualVerifierReviewStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
