use super::*;

use acme_kit::text;

/// Where a zone tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneTierStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ZoneTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ZoneTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a zone tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneTier {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub country: String,
    pub audience: String,
    pub priority: i64,
    pub revision: i64,
    pub segment: String,
    pub namespace: String,
    pub status: ZoneTierStatus,
}

impl ZoneTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneTierStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneTierStatus) -> bool {
        match self.status {
            ZoneTierStatus::Planned => {
                match next {
                    ZoneTierStatus::Scheduled | ZoneTierStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneTierStatus::Scheduled => {
                match next {
                    ZoneTierStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneTierStatus::InProgress => {
                return next == ZoneTierStatus::Cancelled;
            }
            ZoneTierStatus::Completed => {
                return next == ZoneTierStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.revision as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneTier {
        ZoneTier {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            country: self.country.clone(),
            audience: self.audience.clone(),
            priority: self.priority,
            revision: self.revision,
            segment: self.segment.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a zone tier in status planned.
pub fn new_zone_tier(code: &str, label: &str) -> ZoneTier {
    ZoneTier {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        country: "".to_string(),
        audience: "".to_string(),
        priority: 0,
        revision: 0,
        segment: "".to_string(),
        namespace: "".to_string(),
        status: ZoneTierStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_zone_tier("code-859", "label-96");
        assert_eq!(item.status, ZoneTierStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_zone_tier("code-859", "label-96");
        if item.move_to(ZoneTierStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneTierStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_zone_tier("code-859", "label-96");
        item.status = ZoneTierStatus::Cancelled;
        assert!(item.move_to(ZoneTierStatus::Planned).is_err(), "expected move to fail");
    }
}
