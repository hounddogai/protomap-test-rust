use super::*;

use acme_kit::text;

/// Where a usage cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageCycleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl UsageCycleStatus {
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

impl std::fmt::Display for UsageCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageCycle {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub label: String,
    pub updated_at: i64,
    pub expires_at: i64,
    pub ends_at: i64,
    pub rate: f64,
    pub issuer: String,
    pub principal: String,
    pub status: UsageCycleStatus,
}

impl UsageCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageCycleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageCycleStatus) -> bool {
        match self.status {
            UsageCycleStatus::Draft => {
                match next {
                    UsageCycleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageCycleStatus::Pending => {
                return next == UsageCycleStatus::Active || next == UsageCycleStatus::Suspended;
            }
            UsageCycleStatus::Active => {
                match next {
                    UsageCycleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageCycleStatus::Suspended => {
                match next {
                    UsageCycleStatus::Closed => {
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

    pub fn move_to(&mut self, next: UsageCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage cycle {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.ends_at as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageCycle {
        UsageCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            expires_at: self.expires_at,
            ends_at: self.ends_at,
            rate: self.rate,
            issuer: self.issuer.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a usage cycle in status draft.
pub fn new_usage_cycle(code: &str, title: &str) -> UsageCycle {
    UsageCycle {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        label: "".to_string(),
        updated_at: 0,
        expires_at: 0,
        ends_at: 0,
        rate: 0.0,
        issuer: "".to_string(),
        principal: "".to_string(),
        status: UsageCycleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_usage_cycle("code-900", "title-394");
        assert_eq!(item.status, UsageCycleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_usage_cycle("code-900", "title-394");
        if item.move_to(UsageCycleStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageCycleStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_usage_cycle("code-900", "title-394");
        item.status = UsageCycleStatus::Closed;
        assert!(item.move_to(UsageCycleStatus::Draft).is_err(), "expected move to fail");
    }
}
