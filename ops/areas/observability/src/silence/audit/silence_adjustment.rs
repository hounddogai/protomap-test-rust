use super::*;

use acme_kit::text;

/// Where a silence adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SilenceAdjustmentStatus {
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

impl std::fmt::Display for SilenceAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence adjustment as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAdjustment {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub external_ref: String,
    pub priority: i64,
    pub region: String,
    pub rotation_days: i64,
    pub cipher: String,
    pub status: SilenceAdjustmentStatus,
}

impl SilenceAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceAdjustmentStatus) -> bool {
        match self.status {
            SilenceAdjustmentStatus::Planned => {
                return next == SilenceAdjustmentStatus::Completed;
            }
            SilenceAdjustmentStatus::Scheduled => {
                match next {
                    SilenceAdjustmentStatus::InProgress | SilenceAdjustmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceAdjustmentStatus::InProgress => {
                return next == SilenceAdjustmentStatus::Completed || next == SilenceAdjustmentStatus::Cancelled;
            }
            SilenceAdjustmentStatus::Completed => {
                match next {
                    SilenceAdjustmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SilenceAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ends_at as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceAdjustment {
        SilenceAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            external_ref: self.external_ref.clone(),
            priority: self.priority,
            region: self.region.clone(),
            rotation_days: self.rotation_days,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a silence adjustment in status planned.
pub fn new_silence_adjustment(code: &str, label: &str) -> SilenceAdjustment {
    SilenceAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        external_ref: "".to_string(),
        priority: 0,
        region: "".to_string(),
        rotation_days: 0,
        cipher: "".to_string(),
        status: SilenceAdjustmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_silence_adjustment("code-630", "label-130");
        assert_eq!(item.status, SilenceAdjustmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_silence_adjustment("code-630", "label-130");
        if item.move_to(SilenceAdjustmentStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceAdjustmentStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_silence_adjustment("code-630", "label-130");
        item.status = SilenceAdjustmentStatus::Cancelled;
        assert!(
            item.move_to(SilenceAdjustmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
