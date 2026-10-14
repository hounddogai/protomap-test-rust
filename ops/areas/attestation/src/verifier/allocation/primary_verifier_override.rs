use super::*;

use acme_kit::text;

/// Where a primary verifier override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryVerifierOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryVerifierOverrideStatus {
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

impl std::fmt::Display for PrimaryVerifierOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary verifier override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierOverride {
    pub code: String,
    pub title: String,
    pub region: String,
    pub ceiling: f64,
    pub ends_at: i64,
    pub score: f64,
    pub quantity: i64,
    pub label: String,
    pub expires_at: i64,
    pub country: String,
    pub locale: String,
    pub status: PrimaryVerifierOverrideStatus,
}

impl PrimaryVerifierOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryVerifierOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryVerifierOverrideStatus) -> bool {
        match self.status {
            PrimaryVerifierOverrideStatus::Open => {
                return next == PrimaryVerifierOverrideStatus::Resolved
                    || next == PrimaryVerifierOverrideStatus::Archived;
            }
            PrimaryVerifierOverrideStatus::InReview => {
                match next {
                    PrimaryVerifierOverrideStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryVerifierOverrideStatus::Resolved => {
                return next == PrimaryVerifierOverrideStatus::Archived;
            }
            PrimaryVerifierOverrideStatus::Reopened => {
                return next == PrimaryVerifierOverrideStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryVerifierOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary verifier override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary verifier override", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ceiling - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryVerifierOverride {
        PrimaryVerifierOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            ceiling: self.ceiling,
            ends_at: self.ends_at,
            score: self.score,
            quantity: self.quantity,
            label: self.label.clone(),
            expires_at: self.expires_at,
            country: self.country.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary verifier override in status open.
pub fn new_primary_verifier_override(code: &str, title: &str) -> PrimaryVerifierOverride {
    PrimaryVerifierOverride {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        ceiling: 0.0,
        ends_at: 0,
        score: 0.0,
        quantity: 0,
        label: "".to_string(),
        expires_at: 0,
        country: "".to_string(),
        locale: "".to_string(),
        status: PrimaryVerifierOverrideStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_primary_verifier_override("code-270", "title-837");
        assert_eq!(item.status, PrimaryVerifierOverrideStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_primary_verifier_override("code-270", "title-837");
        if item.move_to(PrimaryVerifierOverrideStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryVerifierOverrideStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_primary_verifier_override("code-270", "title-837");
        item.status = PrimaryVerifierOverrideStatus::Archived;
        assert!(
            item.move_to(PrimaryVerifierOverrideStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
