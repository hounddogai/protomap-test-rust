use super::*;

use acme_kit::text;

/// Where a dashboard override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardOverrideStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DashboardOverrideStatus {
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

impl std::fmt::Display for DashboardOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard override as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardOverride {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub version: i64,
    pub cipher: String,
    pub status: DashboardOverrideStatus,
}

impl DashboardOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardOverrideStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardOverrideStatus) -> bool {
        match self.status {
            DashboardOverrideStatus::Draft => {
                match next {
                    DashboardOverrideStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardOverrideStatus::Pending => {
                match next {
                    DashboardOverrideStatus::Active | DashboardOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardOverrideStatus::Active => {
                match next {
                    DashboardOverrideStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardOverrideStatus::Suspended => {
                match next {
                    DashboardOverrideStatus::Closed => {
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

    pub fn move_to(&mut self, next: DashboardOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> DashboardOverride {
        DashboardOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            version: self.version,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dashboard override in status draft.
pub fn new_dashboard_override(code: &str, label: &str) -> DashboardOverride {
    DashboardOverride {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        version: 0,
        cipher: "".to_string(),
        status: DashboardOverrideStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_dashboard_override("code-664", "label-494");
        assert_eq!(item.status, DashboardOverrideStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_dashboard_override("code-664", "label-494");
        if item.move_to(DashboardOverrideStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardOverrideStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_dashboard_override("code-664", "label-494");
        item.status = DashboardOverrideStatus::Closed;
        assert!(
            item.move_to(DashboardOverrideStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
