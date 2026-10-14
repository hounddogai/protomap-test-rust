use super::*;

use acme_kit::text;

/// Where a legacy reservation adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyReservationAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyReservationAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyReservationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy reservation adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationAdjustment {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub visible: bool,
    pub scope: String,
    pub cluster: String,
    pub status: LegacyReservationAdjustmentStatus,
}

impl LegacyReservationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyReservationAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyReservationAdjustmentStatus) -> bool {
        match self.status {
            LegacyReservationAdjustmentStatus::Created => {
                match next {
                    LegacyReservationAdjustmentStatus::Published | LegacyReservationAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyReservationAdjustmentStatus::Validated => {
                return next == LegacyReservationAdjustmentStatus::Published
                    || next == LegacyReservationAdjustmentStatus::Retired;
            }
            LegacyReservationAdjustmentStatus::Published => {
                return next == LegacyReservationAdjustmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyReservationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy reservation adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> LegacyReservationAdjustment {
        LegacyReservationAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            visible: self.visible,
            scope: self.scope.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a legacy reservation adjustment in status created.
pub fn new_legacy_reservation_adjustment(code: &str, title: &str) -> LegacyReservationAdjustment {
    LegacyReservationAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        visible: false,
        scope: "".to_string(),
        cluster: "".to_string(),
        status: LegacyReservationAdjustmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_reservation_adjustment("code-273", "title-202");
        assert_eq!(item.status, LegacyReservationAdjustmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_legacy_reservation_adjustment("code-273", "title-202");
        if item.move_to(LegacyReservationAdjustmentStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyReservationAdjustmentStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_reservation_adjustment("code-273", "title-202");
        item.status = LegacyReservationAdjustmentStatus::Retired;
        assert!(
            item.move_to(LegacyReservationAdjustmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
