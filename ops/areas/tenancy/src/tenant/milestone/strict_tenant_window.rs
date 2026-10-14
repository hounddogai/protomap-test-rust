use super::*;

use acme_kit::text;

/// Where a strict tenant window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictTenantWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictTenantWindowStatus {
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

impl std::fmt::Display for StrictTenantWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict tenant window as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTenantWindow {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub timezone: String,
    pub fingerprint: String,
    pub namespace: String,
    pub active: bool,
    pub revision: i64,
    pub status: StrictTenantWindowStatus,
}

impl StrictTenantWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictTenantWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictTenantWindowStatus) -> bool {
        match self.status {
            StrictTenantWindowStatus::Draft => {
                return next == StrictTenantWindowStatus::Suspended || next == StrictTenantWindowStatus::Closed;
            }
            StrictTenantWindowStatus::Pending => {
                return next == StrictTenantWindowStatus::Closed;
            }
            StrictTenantWindowStatus::Active => {
                return next == StrictTenantWindowStatus::Suspended;
            }
            StrictTenantWindowStatus::Suspended => {
                return next == StrictTenantWindowStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictTenantWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict tenant window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.amount - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictTenantWindow {
        StrictTenantWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            timezone: self.timezone.clone(),
            fingerprint: self.fingerprint.clone(),
            namespace: self.namespace.clone(),
            active: self.active,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a strict tenant window in status draft.
pub fn new_strict_tenant_window(code: &str, title: &str) -> StrictTenantWindow {
    StrictTenantWindow {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        timezone: "".to_string(),
        fingerprint: "".to_string(),
        namespace: "".to_string(),
        active: false,
        revision: 0,
        status: StrictTenantWindowStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_tenant_window("code-986", "title-4");
        assert_eq!(item.status, StrictTenantWindowStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_strict_tenant_window("code-986", "title-4");
        if item.move_to(StrictTenantWindowStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictTenantWindowStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_tenant_window("code-986", "title-4");
        item.status = StrictTenantWindowStatus::Closed;
        assert!(
            item.move_to(StrictTenantWindowStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
