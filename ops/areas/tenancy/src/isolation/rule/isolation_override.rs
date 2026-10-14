use super::*;

use acme_kit::text;

/// Where a isolation override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationOverrideStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl IsolationOverrideStatus {
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

impl std::fmt::Display for IsolationOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation override as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationOverride {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub updated_at: i64,
    pub issuer: String,
    pub unit: String,
    pub namespace: String,
    pub active: bool,
    pub status: IsolationOverrideStatus,
}

impl IsolationOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationOverrideStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationOverrideStatus) -> bool {
        match self.status {
            IsolationOverrideStatus::Draft => {
                match next {
                    IsolationOverrideStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationOverrideStatus::Pending => {
                match next {
                    IsolationOverrideStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationOverrideStatus::Active => {
                return next == IsolationOverrideStatus::Closed;
            }
            IsolationOverrideStatus::Suspended => {
                match next {
                    IsolationOverrideStatus::Closed => {
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

    pub fn move_to(&mut self, next: IsolationOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation override {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.capacity as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationOverride {
        IsolationOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            updated_at: self.updated_at,
            issuer: self.issuer.clone(),
            unit: self.unit.clone(),
            namespace: self.namespace.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a isolation override in status draft.
pub fn new_isolation_override(code: &str, title: &str) -> IsolationOverride {
    IsolationOverride {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        updated_at: 0,
        issuer: "".to_string(),
        unit: "".to_string(),
        namespace: "".to_string(),
        active: false,
        status: IsolationOverrideStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_isolation_override("code-738", "title-624");
        assert_eq!(item.status, IsolationOverrideStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_isolation_override("code-738", "title-624");
        if item.move_to(IsolationOverrideStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationOverrideStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_isolation_override("code-738", "title-624");
        item.status = IsolationOverrideStatus::Closed;
        assert!(
            item.move_to(IsolationOverrideStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
