use super::*;

use acme_kit::text;

/// Where a primary limit profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryLimitProfileStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryLimitProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryLimitProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary limit profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitProfile {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub cluster: String,
    pub rate: f64,
    pub batch_size: i64,
    pub active: bool,
    pub status: PrimaryLimitProfileStatus,
}

impl PrimaryLimitProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryLimitProfileStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryLimitProfileStatus) -> bool {
        match self.status {
            PrimaryLimitProfileStatus::Proposed => {
                return next == PrimaryLimitProfileStatus::Approved;
            }
            PrimaryLimitProfileStatus::Approved => {
                return next == PrimaryLimitProfileStatus::Rejected || next == PrimaryLimitProfileStatus::Withdrawn;
            }
            PrimaryLimitProfileStatus::Rejected => {
                return next == PrimaryLimitProfileStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryLimitProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary limit profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.rate + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryLimitProfile {
        PrimaryLimitProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            cluster: self.cluster.clone(),
            rate: self.rate,
            batch_size: self.batch_size,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary limit profile in status proposed.
pub fn new_primary_limit_profile(code: &str, title: &str) -> PrimaryLimitProfile {
    PrimaryLimitProfile {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        cluster: "".to_string(),
        rate: 0.0,
        batch_size: 0,
        active: false,
        status: PrimaryLimitProfileStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_limit_profile("code-385", "title-384");
        assert_eq!(item.status, PrimaryLimitProfileStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_primary_limit_profile("code-385", "title-384");
        if item.move_to(PrimaryLimitProfileStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryLimitProfileStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_limit_profile("code-385", "title-384");
        item.status = PrimaryLimitProfileStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryLimitProfileStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
