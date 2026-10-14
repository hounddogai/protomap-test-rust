use super::*;

use acme_kit::text;

/// Where a rolling release plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingReleasePlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingReleasePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingReleasePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling release plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleasePlan {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub issuer: String,
    pub quota_units: i64,
    pub ceiling: f64,
    pub locale: String,
    pub status: RollingReleasePlanStatus,
}

impl RollingReleasePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingReleasePlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingReleasePlanStatus) -> bool {
        match self.status {
            RollingReleasePlanStatus::Created => {
                match next {
                    RollingReleasePlanStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingReleasePlanStatus::Validated => {
                match next {
                    RollingReleasePlanStatus::Published | RollingReleasePlanStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingReleasePlanStatus::Published => {
                match next {
                    RollingReleasePlanStatus::Retired => {
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

    pub fn move_to(&mut self, next: RollingReleasePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling release plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.quota_units as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingReleasePlan {
        RollingReleasePlan {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            issuer: self.issuer.clone(),
            quota_units: self.quota_units,
            ceiling: self.ceiling,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rolling release plan in status created.
pub fn new_rolling_release_plan(code: &str, label: &str) -> RollingReleasePlan {
    RollingReleasePlan {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        issuer: "".to_string(),
        quota_units: 0,
        ceiling: 0.0,
        locale: "".to_string(),
        status: RollingReleasePlanStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rolling_release_plan("code-729", "label-849");
        assert_eq!(item.status, RollingReleasePlanStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_rolling_release_plan("code-729", "label-849");
        if item.move_to(RollingReleasePlanStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingReleasePlanStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rolling_release_plan("code-729", "label-849");
        item.status = RollingReleasePlanStatus::Retired;
        assert!(
            item.move_to(RollingReleasePlanStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
