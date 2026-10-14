use super::*;

use acme_kit::text;

/// Where a global measurement profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalMeasurementProfileStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalMeasurementProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalMeasurementProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global measurement profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementProfile {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub scope: String,
    pub timezone: String,
    pub archived: bool,
    pub threshold: f64,
    pub limit: i64,
    pub quantity: i64,
    pub status: GlobalMeasurementProfileStatus,
}

impl GlobalMeasurementProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalMeasurementProfileStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalMeasurementProfileStatus) -> bool {
        match self.status {
            GlobalMeasurementProfileStatus::Created => {
                return next == GlobalMeasurementProfileStatus::Retired;
            }
            GlobalMeasurementProfileStatus::Validated => {
                return next == GlobalMeasurementProfileStatus::Published
                    || next == GlobalMeasurementProfileStatus::Retired;
            }
            GlobalMeasurementProfileStatus::Published => {
                return next == GlobalMeasurementProfileStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalMeasurementProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global measurement profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.quantity as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalMeasurementProfile {
        GlobalMeasurementProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            scope: self.scope.clone(),
            timezone: self.timezone.clone(),
            archived: self.archived,
            threshold: self.threshold,
            limit: self.limit,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a global measurement profile in status created.
pub fn new_global_measurement_profile(code: &str, title: &str) -> GlobalMeasurementProfile {
    GlobalMeasurementProfile {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        scope: "".to_string(),
        timezone: "".to_string(),
        archived: false,
        threshold: 0.0,
        limit: 0,
        quantity: 0,
        status: GlobalMeasurementProfileStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_global_measurement_profile("code-87", "title-22");
        assert_eq!(item.status, GlobalMeasurementProfileStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_global_measurement_profile("code-87", "title-22");
        if item.move_to(GlobalMeasurementProfileStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalMeasurementProfileStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_global_measurement_profile("code-87", "title-22");
        item.status = GlobalMeasurementProfileStatus::Retired;
        assert!(
            item.move_to(GlobalMeasurementProfileStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
