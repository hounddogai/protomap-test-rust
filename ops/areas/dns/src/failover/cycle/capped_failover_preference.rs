use super::*;

use acme_kit::text;

/// Where a capped failover preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedFailoverPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedFailoverPreferenceStatus {
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

impl std::fmt::Display for CappedFailoverPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped failover preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverPreference {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub country: String,
    pub ceiling: f64,
    pub issuer: String,
    pub category: String,
    pub status: CappedFailoverPreferenceStatus,
}

impl CappedFailoverPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedFailoverPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedFailoverPreferenceStatus) -> bool {
        match self.status {
            CappedFailoverPreferenceStatus::Draft => {
                match next {
                    CappedFailoverPreferenceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedFailoverPreferenceStatus::Pending => {
                return next == CappedFailoverPreferenceStatus::Active;
            }
            CappedFailoverPreferenceStatus::Active => {
                match next {
                    CappedFailoverPreferenceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedFailoverPreferenceStatus::Suspended => {
                match next {
                    CappedFailoverPreferenceStatus::Closed => {
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

    pub fn move_to(&mut self, next: CappedFailoverPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped failover preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> CappedFailoverPreference {
        CappedFailoverPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            country: self.country.clone(),
            ceiling: self.ceiling,
            issuer: self.issuer.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a capped failover preference in status draft.
pub fn new_capped_failover_preference(code: &str, name: &str) -> CappedFailoverPreference {
    CappedFailoverPreference {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        country: "".to_string(),
        ceiling: 0.0,
        issuer: "".to_string(),
        category: "".to_string(),
        status: CappedFailoverPreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_failover_preference("code-311", "name-322");
        assert_eq!(item.status, CappedFailoverPreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_capped_failover_preference("code-311", "name-322");
        if item.move_to(CappedFailoverPreferenceStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedFailoverPreferenceStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_failover_preference("code-311", "name-322");
        item.status = CappedFailoverPreferenceStatus::Closed;
        assert!(
            item.move_to(CappedFailoverPreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
