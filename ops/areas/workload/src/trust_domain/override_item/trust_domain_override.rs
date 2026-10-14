use super::*;

use acme_kit::text;

/// Where a trust domain override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainOverrideStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TrustDomainOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TrustDomainOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One trust domain override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainOverride {
    pub code: String,
    pub title: String,
    pub source: String,
    pub archived: bool,
    pub floor_value: f64,
    pub minimum: f64,
    pub grace_days: i64,
    pub ceiling: f64,
    pub status: TrustDomainOverrideStatus,
}

impl TrustDomainOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainOverrideStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainOverrideStatus) -> bool {
        match self.status {
            TrustDomainOverrideStatus::Created => {
                return next == TrustDomainOverrideStatus::Validated;
            }
            TrustDomainOverrideStatus::Validated => {
                return next == TrustDomainOverrideStatus::Published || next == TrustDomainOverrideStatus::Retired;
            }
            TrustDomainOverrideStatus::Published => {
                match next {
                    TrustDomainOverrideStatus::Retired => {
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

    pub fn move_to(&mut self, next: TrustDomainOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ceiling - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainOverride {
        TrustDomainOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            archived: self.archived,
            floor_value: self.floor_value,
            minimum: self.minimum,
            grace_days: self.grace_days,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a trust domain override in status created.
pub fn new_trust_domain_override(code: &str, title: &str) -> TrustDomainOverride {
    TrustDomainOverride {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        archived: false,
        floor_value: 0.0,
        minimum: 0.0,
        grace_days: 0,
        ceiling: 0.0,
        status: TrustDomainOverrideStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_trust_domain_override("code-864", "title-901");
        assert_eq!(item.status, TrustDomainOverrideStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_trust_domain_override("code-864", "title-901");
        if item.move_to(TrustDomainOverrideStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainOverrideStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_trust_domain_override("code-864", "title-901");
        item.status = TrustDomainOverrideStatus::Retired;
        assert!(
            item.move_to(TrustDomainOverrideStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
