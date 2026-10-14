use super::*;

use acme_kit::text;

/// Where a pending slo profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSloProfileStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingSloProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingSloProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending slo profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloProfile {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub minimum: f64,
    pub visible: bool,
    pub status: PendingSloProfileStatus,
}

impl PendingSloProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSloProfileStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSloProfileStatus) -> bool {
        match self.status {
            PendingSloProfileStatus::Created => {
                match next {
                    PendingSloProfileStatus::Published | PendingSloProfileStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSloProfileStatus::Validated => {
                return next == PendingSloProfileStatus::Retired;
            }
            PendingSloProfileStatus::Published => {
                match next {
                    PendingSloProfileStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingSloProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending slo profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.minimum * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingSloProfile {
        PendingSloProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            minimum: self.minimum,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending slo profile in status created.
pub fn new_pending_slo_profile(code: &str, label: &str) -> PendingSloProfile {
    PendingSloProfile {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        minimum: 0.0,
        visible: false,
        status: PendingSloProfileStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_slo_profile("code-940", "label-50");
        assert_eq!(item.status, PendingSloProfileStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_pending_slo_profile("code-940", "label-50");
        if item.move_to(PendingSloProfileStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingSloProfileStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_slo_profile("code-940", "label-50");
        item.status = PendingSloProfileStatus::Retired;
        assert!(
            item.move_to(PendingSloProfileStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
