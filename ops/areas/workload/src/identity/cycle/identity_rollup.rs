use super::*;

use acme_kit::text;

/// Where a identity rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl IdentityRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for IdentityRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity rollup as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRollup {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub version: i64,
    pub category: String,
    pub unit: String,
    pub tenant_id: String,
    pub region: String,
    pub weight: f64,
    pub pinned: bool,
    pub status: IdentityRollupStatus,
}

impl IdentityRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityRollupStatus) -> bool {
        match self.status {
            IdentityRollupStatus::Draft => {
                return next == IdentityRollupStatus::Active;
            }
            IdentityRollupStatus::Pending => {
                match next {
                    IdentityRollupStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityRollupStatus::Active => {
                return next == IdentityRollupStatus::Suspended;
            }
            IdentityRollupStatus::Suspended => {
                match next {
                    IdentityRollupStatus::Closed => {
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

    pub fn move_to(&mut self, next: IdentityRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.amount + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityRollup {
        IdentityRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            version: self.version,
            category: self.category.clone(),
            unit: self.unit.clone(),
            tenant_id: self.tenant_id.clone(),
            region: self.region.clone(),
            weight: self.weight,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a identity rollup in status draft.
pub fn new_identity_rollup(code: &str, name: &str) -> IdentityRollup {
    IdentityRollup {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        version: 0,
        category: "".to_string(),
        unit: "".to_string(),
        tenant_id: "".to_string(),
        region: "".to_string(),
        weight: 0.0,
        pinned: false,
        status: IdentityRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_identity_rollup("code-429", "name-685");
        assert_eq!(item.status, IdentityRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_identity_rollup("code-429", "name-685");
        if item.move_to(IdentityRollupStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityRollupStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_identity_rollup("code-429", "name-685");
        item.status = IdentityRollupStatus::Closed;
        assert!(
            item.move_to(IdentityRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
