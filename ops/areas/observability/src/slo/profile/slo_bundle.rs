use super::*;

use acme_kit::text;

/// Where a slo bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloBundleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SloBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SloBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One slo bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloBundle {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub created_at: i64,
    pub audience: String,
    pub ceiling: f64,
    pub reviewer: String,
    pub notes: String,
    pub quantity: i64,
    pub status: SloBundleStatus,
}

impl SloBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloBundleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloBundleStatus) -> bool {
        match self.status {
            SloBundleStatus::Requested => {
                return next == SloBundleStatus::Granted || next == SloBundleStatus::Revoked;
            }
            SloBundleStatus::Granted => {
                match next {
                    SloBundleStatus::Revoked | SloBundleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloBundleStatus::Revoked => {
                return next == SloBundleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "slo bundle", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.quantity as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloBundle {
        SloBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            created_at: self.created_at,
            audience: self.audience.clone(),
            ceiling: self.ceiling,
            reviewer: self.reviewer.clone(),
            notes: self.notes.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a slo bundle in status requested.
pub fn new_slo_bundle(code: &str, label: &str) -> SloBundle {
    SloBundle {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        created_at: 0,
        audience: "".to_string(),
        ceiling: 0.0,
        reviewer: "".to_string(),
        notes: "".to_string(),
        quantity: 0,
        status: SloBundleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_slo_bundle("code-450", "label-376");
        assert_eq!(item.status, SloBundleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_slo_bundle("code-450", "label-376");
        if item.move_to(SloBundleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloBundleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_slo_bundle("code-450", "label-376");
        item.status = SloBundleStatus::Expired;
        assert!(
            item.move_to(SloBundleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
