use super::*;

use acme_kit::text;

/// Where a rolling plan mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPlanMappingStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingPlanMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingPlanMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling plan mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanMapping {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub timezone: String,
    pub limit: i64,
    pub status: RollingPlanMappingStatus,
}

impl RollingPlanMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPlanMappingStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPlanMappingStatus) -> bool {
        match self.status {
            RollingPlanMappingStatus::Created => {
                match next {
                    RollingPlanMappingStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPlanMappingStatus::Validated => {
                return next == RollingPlanMappingStatus::Published || next == RollingPlanMappingStatus::Retired;
            }
            RollingPlanMappingStatus::Published => {
                return next == RollingPlanMappingStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingPlanMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling plan mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> RollingPlanMapping {
        RollingPlanMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            timezone: self.timezone.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling plan mapping in status created.
pub fn new_rolling_plan_mapping(code: &str, title: &str) -> RollingPlanMapping {
    RollingPlanMapping {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        timezone: "".to_string(),
        limit: 0,
        status: RollingPlanMappingStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rolling_plan_mapping("code-360", "title-341");
        assert_eq!(item.status, RollingPlanMappingStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_rolling_plan_mapping("code-360", "title-341");
        if item.move_to(RollingPlanMappingStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPlanMappingStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rolling_plan_mapping("code-360", "title-341");
        item.status = RollingPlanMappingStatus::Retired;
        assert!(
            item.move_to(RollingPlanMappingStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
