use super::*;

use acme_kit::text;

/// Where a global rollout variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRolloutVarianceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalRolloutVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalRolloutVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global rollout variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutVariance {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub phone: String,
    pub visible: bool,
    pub rotation_days: i64,
    pub reviewer: String,
    pub checksum: String,
    pub quota_units: i64,
    pub version: i64,
    pub status: GlobalRolloutVarianceStatus,
}

impl GlobalRolloutVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRolloutVarianceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRolloutVarianceStatus) -> bool {
        match self.status {
            GlobalRolloutVarianceStatus::Created => {
                return next == GlobalRolloutVarianceStatus::Published;
            }
            GlobalRolloutVarianceStatus::Validated => {
                match next {
                    GlobalRolloutVarianceStatus::Published | GlobalRolloutVarianceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRolloutVarianceStatus::Published => {
                return next == GlobalRolloutVarianceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalRolloutVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global rollout variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.version as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalRolloutVariance {
        GlobalRolloutVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            phone: self.phone.clone(),
            visible: self.visible,
            rotation_days: self.rotation_days,
            reviewer: self.reviewer.clone(),
            checksum: self.checksum.clone(),
            quota_units: self.quota_units,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global rollout variance in status created.
pub fn new_global_rollout_variance(code: &str, label: &str) -> GlobalRolloutVariance {
    GlobalRolloutVariance {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        phone: "".to_string(),
        visible: false,
        rotation_days: 0,
        reviewer: "".to_string(),
        checksum: "".to_string(),
        quota_units: 0,
        version: 0,
        status: GlobalRolloutVarianceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_global_rollout_variance("code-445", "label-525");
        assert_eq!(item.status, GlobalRolloutVarianceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_global_rollout_variance("code-445", "label-525");
        if item.move_to(GlobalRolloutVarianceStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalRolloutVarianceStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_global_rollout_variance("code-445", "label-525");
        item.status = GlobalRolloutVarianceStatus::Retired;
        assert!(
            item.move_to(GlobalRolloutVarianceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
