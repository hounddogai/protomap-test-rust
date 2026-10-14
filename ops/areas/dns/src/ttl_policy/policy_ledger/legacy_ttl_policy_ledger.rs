use super::*;

use acme_kit::text;

/// Where a legacy ttl policy ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyTtlPolicyLedgerStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyTtlPolicyLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyTtlPolicyLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy ttl policy ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTtlPolicyLedger {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub country: String,
    pub audience: String,
    pub margin: f64,
    pub quota_units: i64,
    pub version: i64,
    pub tags: Vec<String>,
    pub rotation_days: i64,
    pub title: String,
    pub status: LegacyTtlPolicyLedgerStatus,
}

impl LegacyTtlPolicyLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyTtlPolicyLedgerStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyTtlPolicyLedgerStatus) -> bool {
        match self.status {
            LegacyTtlPolicyLedgerStatus::Created => {
                return next == LegacyTtlPolicyLedgerStatus::Published || next == LegacyTtlPolicyLedgerStatus::Retired;
            }
            LegacyTtlPolicyLedgerStatus::Validated => {
                match next {
                    LegacyTtlPolicyLedgerStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyTtlPolicyLedgerStatus::Published => {
                match next {
                    LegacyTtlPolicyLedgerStatus::Retired => {
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

    pub fn move_to(&mut self, next: LegacyTtlPolicyLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy ttl policy ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.priority as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyTtlPolicyLedger {
        LegacyTtlPolicyLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            country: self.country.clone(),
            audience: self.audience.clone(),
            margin: self.margin,
            quota_units: self.quota_units,
            version: self.version,
            tags: self.tags.clone(),
            rotation_days: self.rotation_days,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a legacy ttl policy ledger in status created.
pub fn new_legacy_ttl_policy_ledger(code: &str, name: &str) -> LegacyTtlPolicyLedger {
    LegacyTtlPolicyLedger {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        country: "".to_string(),
        audience: "".to_string(),
        margin: 0.0,
        quota_units: 0,
        version: 0,
        tags: Vec::new(),
        rotation_days: 0,
        title: "".to_string(),
        status: LegacyTtlPolicyLedgerStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_ttl_policy_ledger("code-819", "name-931");
        assert_eq!(item.status, LegacyTtlPolicyLedgerStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_legacy_ttl_policy_ledger("code-819", "name-931");
        if item.move_to(LegacyTtlPolicyLedgerStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyTtlPolicyLedgerStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_ttl_policy_ledger("code-819", "name-931");
        item.status = LegacyTtlPolicyLedgerStatus::Retired;
        assert!(
            item.move_to(LegacyTtlPolicyLedgerStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
