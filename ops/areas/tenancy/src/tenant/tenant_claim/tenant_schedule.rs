use super::*;

use acme_kit::text;

/// Where a tenant schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantScheduleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TenantScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TenantScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantSchedule {
    pub code: String,
    pub label: String,
    pub description: String,
    pub revision: i64,
    pub cluster: String,
    pub rate: f64,
    pub phone: String,
    pub status: TenantScheduleStatus,
}

impl TenantSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantScheduleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantScheduleStatus) -> bool {
        match self.status {
            TenantScheduleStatus::Created => {
                return next == TenantScheduleStatus::Validated || next == TenantScheduleStatus::Retired;
            }
            TenantScheduleStatus::Validated => {
                return next == TenantScheduleStatus::Published || next == TenantScheduleStatus::Retired;
            }
            TenantScheduleStatus::Published => {
                match next {
                    TenantScheduleStatus::Retired => {
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

    pub fn move_to(&mut self, next: TenantScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.revision as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantSchedule {
        TenantSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            revision: self.revision,
            cluster: self.cluster.clone(),
            rate: self.rate,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a tenant schedule in status created.
pub fn new_tenant_schedule(code: &str, label: &str) -> TenantSchedule {
    TenantSchedule {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        revision: 0,
        cluster: "".to_string(),
        rate: 0.0,
        phone: "".to_string(),
        status: TenantScheduleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tenant_schedule("code-915", "label-854");
        assert_eq!(item.status, TenantScheduleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tenant_schedule("code-915", "label-854");
        if item.move_to(TenantScheduleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantScheduleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tenant_schedule("code-915", "label-854");
        item.status = TenantScheduleStatus::Retired;
        assert!(
            item.move_to(TenantScheduleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
