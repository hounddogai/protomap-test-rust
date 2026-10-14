use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a verifier review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl VerifierReviewStatus {
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

impl std::fmt::Display for VerifierReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier review as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReview {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub cluster: String,
    pub attributes: BTreeMap<String, String>,
    pub ttl_seconds: i64,
    pub cost: f64,
    pub tags: Vec<String>,
    pub grace_days: i64,
    pub contact_email: String,
    pub revision: i64,
    pub status: VerifierReviewStatus,
}

impl VerifierReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierReviewStatus) -> bool {
        match self.status {
            VerifierReviewStatus::New => {
                match next {
                    VerifierReviewStatus::Running | VerifierReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierReviewStatus::Queued => {
                match next {
                    VerifierReviewStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierReviewStatus::Running => {
                return next == VerifierReviewStatus::Done || next == VerifierReviewStatus::Failed;
            }
            VerifierReviewStatus::Done => {
                return next == VerifierReviewStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.revision as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierReview {
        VerifierReview {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            cluster: self.cluster.clone(),
            attributes: self.attributes.clone(),
            ttl_seconds: self.ttl_seconds,
            cost: self.cost,
            tags: self.tags.clone(),
            grace_days: self.grace_days,
            contact_email: self.contact_email.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a verifier review in status new.
pub fn new_verifier_review(code: &str, name: &str) -> VerifierReview {
    VerifierReview {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        cluster: "".to_string(),
        attributes: BTreeMap::new(),
        ttl_seconds: 0,
        cost: 0.0,
        tags: Vec::new(),
        grace_days: 0,
        contact_email: "".to_string(),
        revision: 0,
        status: VerifierReviewStatus::New,
    }
}
