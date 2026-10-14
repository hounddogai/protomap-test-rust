use super::*;

use acme_kit::text;

/// Where a manual timeline adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualTimelineAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualTimelineAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualTimelineAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual timeline adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTimelineAdjustment {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub active: bool,
    pub port: i64,
    pub principal: String,
    pub locale: String,
    pub grace_days: i64,
    pub ratio: f64,
    pub weight: f64,
    pub amount: f64,
    pub status: ManualTimelineAdjustmentStatus,
}

impl ManualTimelineAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualTimelineAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualTimelineAdjustmentStatus) -> bool {
        match self.status {
            ManualTimelineAdjustmentStatus::Created => {
                match next {
                    ManualTimelineAdjustmentStatus::Published | ManualTimelineAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTimelineAdjustmentStatus::Validated => {
                match next {
                    ManualTimelineAdjustmentStatus::Published | ManualTimelineAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTimelineAdjustmentStatus::Published => {
                match next {
                    ManualTimelineAdjustmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: ManualTimelineAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual timeline adjustment {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.updated_at as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualTimelineAdjustment {
        ManualTimelineAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            active: self.active,
            port: self.port,
            principal: self.principal.clone(),
            locale: self.locale.clone(),
            grace_days: self.grace_days,
            ratio: self.ratio,
            weight: self.weight,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a manual timeline adjustment in status created.
pub fn new_manual_timeline_adjustment(code: &str, label: &str) -> ManualTimelineAdjustment {
    ManualTimelineAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        active: false,
        port: 0,
        principal: "".to_string(),
        locale: "".to_string(),
        grace_days: 0,
        ratio: 0.0,
        weight: 0.0,
        amount: 0.0,
        status: ManualTimelineAdjustmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_timeline_adjustment("code-983", "label-216");
        assert_eq!(item.status, ManualTimelineAdjustmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_manual_timeline_adjustment("code-983", "label-216");
        if item.move_to(ManualTimelineAdjustmentStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualTimelineAdjustmentStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_timeline_adjustment("code-983", "label-216");
        item.status = ManualTimelineAdjustmentStatus::Retired;
        assert!(
            item.move_to(ManualTimelineAdjustmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
