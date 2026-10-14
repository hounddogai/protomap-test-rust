use super::*;

use acme_kit::text;

/// Where a legacy trust domain assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyTrustDomainAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyTrustDomainAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyTrustDomainAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy trust domain assignment as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainAssignment {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub window_days: i64,
    pub source: String,
    pub status: LegacyTrustDomainAssignmentStatus,
}

impl LegacyTrustDomainAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyTrustDomainAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyTrustDomainAssignmentStatus) -> bool {
        match self.status {
            LegacyTrustDomainAssignmentStatus::Created => {
                return next == LegacyTrustDomainAssignmentStatus::Published
                    || next == LegacyTrustDomainAssignmentStatus::Retired;
            }
            LegacyTrustDomainAssignmentStatus::Validated => {
                match next {
                    LegacyTrustDomainAssignmentStatus::Published | LegacyTrustDomainAssignmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyTrustDomainAssignmentStatus::Published => {
                match next {
                    LegacyTrustDomainAssignmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: LegacyTrustDomainAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy trust domain assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.window_days as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyTrustDomainAssignment {
        LegacyTrustDomainAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            window_days: self.window_days,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a legacy trust domain assignment in status created.
pub fn new_legacy_trust_domain_assignment(code: &str, title: &str) -> LegacyTrustDomainAssignment {
    LegacyTrustDomainAssignment {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        window_days: 0,
        source: "".to_string(),
        status: LegacyTrustDomainAssignmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_trust_domain_assignment("code-375", "title-910");
        assert_eq!(item.status, LegacyTrustDomainAssignmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_legacy_trust_domain_assignment("code-375", "title-910");
        if item.move_to(LegacyTrustDomainAssignmentStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyTrustDomainAssignmentStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_trust_domain_assignment("code-375", "title-910");
        item.status = LegacyTrustDomainAssignmentStatus::Retired;
        assert!(
            item.move_to(LegacyTrustDomainAssignmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
