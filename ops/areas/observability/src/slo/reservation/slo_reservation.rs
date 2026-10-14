use super::*;

use acme_kit::text;

/// Where a slo reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SloReservationStatus {
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

impl std::fmt::Display for SloReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One slo reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloReservation {
    pub code: String,
    pub label: String,
    pub host: String,
    pub title: String,
    pub rotation_days: i64,
    pub tags: Vec<String>,
    pub key_id: String,
    pub category: String,
    pub version: i64,
    pub archived: bool,
    pub source: String,
    pub status: SloReservationStatus,
}

impl SloReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloReservationStatus) -> bool {
        match self.status {
            SloReservationStatus::Idle => {
                return next == SloReservationStatus::Warming;
            }
            SloReservationStatus::Warming => {
                return next == SloReservationStatus::Ready || next == SloReservationStatus::Draining;
            }
            SloReservationStatus::Ready => {
                return next == SloReservationStatus::Draining || next == SloReservationStatus::Stopped;
            }
            SloReservationStatus::Draining => {
                match next {
                    SloReservationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SloReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo reservation {} cannot move from {} to {}",
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

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.version as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloReservation {
        SloReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            tags: self.tags.clone(),
            key_id: self.key_id.clone(),
            category: self.category.clone(),
            version: self.version,
            archived: self.archived,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a slo reservation in status idle.
pub fn new_slo_reservation(code: &str, label: &str) -> SloReservation {
    SloReservation {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        title: "".to_string(),
        rotation_days: 0,
        tags: Vec::new(),
        key_id: "".to_string(),
        category: "".to_string(),
        version: 0,
        archived: false,
        source: "".to_string(),
        status: SloReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_slo_reservation("code-911", "label-673");
        assert_eq!(item.status, SloReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_slo_reservation("code-911", "label-673");
        if item.move_to(SloReservationStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloReservationStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_slo_reservation("code-911", "label-673");
        item.status = SloReservationStatus::Stopped;
        assert!(
            item.move_to(SloReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
