use super::*;

use acme_kit::text;

/// Where a federation override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationOverrideStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FederationOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FederationOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a federation override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationOverride {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub limit: i64,
    pub weight: f64,
    pub grace_days: i64,
    pub status: FederationOverrideStatus,
}

impl FederationOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationOverrideStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationOverrideStatus) -> bool {
        match self.status {
            FederationOverrideStatus::Created => {
                match next {
                    FederationOverrideStatus::Validated | FederationOverrideStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationOverrideStatus::Validated => {
                return next == FederationOverrideStatus::Retired;
            }
            FederationOverrideStatus::Published => {
                return next == FederationOverrideStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.limit as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationOverride {
        FederationOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            limit: self.limit,
            weight: self.weight,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a federation override in status created.
pub fn new_federation_override(code: &str, title: &str) -> FederationOverride {
    FederationOverride {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        limit: 0,
        weight: 0.0,
        grace_days: 0,
        status: FederationOverrideStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_federation_override("code-913", "title-258");
        assert_eq!(item.status, FederationOverrideStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_federation_override("code-913", "title-258");
        if item.move_to(FederationOverrideStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationOverrideStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_federation_override("code-913", "title-258");
        item.status = FederationOverrideStatus::Retired;
        assert!(
            item.move_to(FederationOverrideStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
