use super::*;

use acme_kit::text;

/// Where a regional zone checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalZoneCheckpointStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalZoneCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RegionalZoneCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional zone checkpoint as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneCheckpoint {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub limit: i64,
    pub issuer: String,
    pub retries: i64,
    pub archived: bool,
    pub status: RegionalZoneCheckpointStatus,
}

impl RegionalZoneCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalZoneCheckpointStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalZoneCheckpointStatus) -> bool {
        match self.status {
            RegionalZoneCheckpointStatus::Idle => {
                match next {
                    RegionalZoneCheckpointStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalZoneCheckpointStatus::Warming => {
                match next {
                    RegionalZoneCheckpointStatus::Ready | RegionalZoneCheckpointStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalZoneCheckpointStatus::Ready => {
                return next == RegionalZoneCheckpointStatus::Draining;
            }
            RegionalZoneCheckpointStatus::Draining => {
                return next == RegionalZoneCheckpointStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalZoneCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional zone checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.retries as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalZoneCheckpoint {
        RegionalZoneCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            limit: self.limit,
            issuer: self.issuer.clone(),
            retries: self.retries,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a regional zone checkpoint in status idle.
pub fn new_regional_zone_checkpoint(code: &str, label: &str) -> RegionalZoneCheckpoint {
    RegionalZoneCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        limit: 0,
        issuer: "".to_string(),
        retries: 0,
        archived: false,
        status: RegionalZoneCheckpointStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_zone_checkpoint("code-368", "label-296");
        assert_eq!(item.status, RegionalZoneCheckpointStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_regional_zone_checkpoint("code-368", "label-296");
        if item.move_to(RegionalZoneCheckpointStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalZoneCheckpointStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_zone_checkpoint("code-368", "label-296");
        item.status = RegionalZoneCheckpointStatus::Stopped;
        assert!(
            item.move_to(RegionalZoneCheckpointStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
