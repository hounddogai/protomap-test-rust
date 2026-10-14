use super::*;

use acme_kit::text;

/// Where a rolling tenant rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTenantRollupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingTenantRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RollingTenantRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling tenant rollup as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantRollup {
    pub code: String,
    pub label: String,
    pub host: String,
    pub margin: f64,
    pub reviewer: String,
    pub namespace: String,
    pub name: String,
    pub external_ref: String,
    pub status: RollingTenantRollupStatus,
}

impl RollingTenantRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTenantRollupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTenantRollupStatus) -> bool {
        match self.status {
            RollingTenantRollupStatus::Planned => {
                match next {
                    RollingTenantRollupStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTenantRollupStatus::Scheduled => {
                return next == RollingTenantRollupStatus::InProgress;
            }
            RollingTenantRollupStatus::InProgress => {
                match next {
                    RollingTenantRollupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTenantRollupStatus::Completed => {
                match next {
                    RollingTenantRollupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RollingTenantRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling tenant rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling tenant rollup", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> RollingTenantRollup {
        RollingTenantRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            margin: self.margin,
            reviewer: self.reviewer.clone(),
            namespace: self.namespace.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rolling tenant rollup in status planned.
pub fn new_rolling_tenant_rollup(code: &str, label: &str) -> RollingTenantRollup {
    RollingTenantRollup {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        margin: 0.0,
        reviewer: "".to_string(),
        namespace: "".to_string(),
        name: "".to_string(),
        external_ref: "".to_string(),
        status: RollingTenantRollupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_tenant_rollup("code-404", "label-330");
        assert_eq!(item.status, RollingTenantRollupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_rolling_tenant_rollup("code-404", "label-330");
        if item.move_to(RollingTenantRollupStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingTenantRollupStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_tenant_rollup("code-404", "label-330");
        item.status = RollingTenantRollupStatus::Cancelled;
        assert!(
            item.move_to(RollingTenantRollupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
