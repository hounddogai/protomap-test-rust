use super::*;

use acme_kit::text;

/// Where a lenient federation mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientFederationMappingStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientFederationMappingStatus {
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

impl std::fmt::Display for LenientFederationMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient federation mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFederationMapping {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub locale: String,
    pub approved: bool,
    pub threshold: f64,
    pub rotation_days: i64,
    pub maximum: f64,
    pub attempts: i64,
    pub category: String,
    pub status: LenientFederationMappingStatus,
}

impl LenientFederationMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientFederationMappingStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientFederationMappingStatus) -> bool {
        match self.status {
            LenientFederationMappingStatus::Open => {
                match next {
                    LenientFederationMappingStatus::InReview | LenientFederationMappingStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientFederationMappingStatus::InReview => {
                match next {
                    LenientFederationMappingStatus::Resolved | LenientFederationMappingStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientFederationMappingStatus::Resolved => {
                match next {
                    LenientFederationMappingStatus::Reopened | LenientFederationMappingStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientFederationMappingStatus::Reopened => {
                match next {
                    LenientFederationMappingStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientFederationMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient federation mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.rotation_days as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientFederationMapping {
        LenientFederationMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            locale: self.locale.clone(),
            approved: self.approved,
            threshold: self.threshold,
            rotation_days: self.rotation_days,
            maximum: self.maximum,
            attempts: self.attempts,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a lenient federation mapping in status open.
pub fn new_lenient_federation_mapping(code: &str, title: &str) -> LenientFederationMapping {
    LenientFederationMapping {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        locale: "".to_string(),
        approved: false,
        threshold: 0.0,
        rotation_days: 0,
        maximum: 0.0,
        attempts: 0,
        category: "".to_string(),
        status: LenientFederationMappingStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_federation_mapping("code-516", "title-176");
        assert_eq!(item.status, LenientFederationMappingStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_lenient_federation_mapping("code-516", "title-176");
        if item.move_to(LenientFederationMappingStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientFederationMappingStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_federation_mapping("code-516", "title-176");
        item.status = LenientFederationMappingStatus::Archived;
        assert!(
            item.move_to(LenientFederationMappingStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
