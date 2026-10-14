use super::*;

use acme_kit::text;

/// Where a capped sink adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedSinkAdjustmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedSinkAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for CappedSinkAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped sink adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkAdjustment {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub source: String,
    pub minimum: f64,
    pub title: String,
    pub timezone: String,
    pub attempts: i64,
    pub cost: f64,
    pub status: CappedSinkAdjustmentStatus,
}

impl CappedSinkAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedSinkAdjustmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedSinkAdjustmentStatus) -> bool {
        match self.status {
            CappedSinkAdjustmentStatus::Draft => {
                return next == CappedSinkAdjustmentStatus::Pending || next == CappedSinkAdjustmentStatus::Active;
            }
            CappedSinkAdjustmentStatus::Pending => {
                return next == CappedSinkAdjustmentStatus::Active || next == CappedSinkAdjustmentStatus::Closed;
            }
            CappedSinkAdjustmentStatus::Active => {
                return next == CappedSinkAdjustmentStatus::Suspended || next == CappedSinkAdjustmentStatus::Closed;
            }
            CappedSinkAdjustmentStatus::Suspended => {
                return next == CappedSinkAdjustmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedSinkAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped sink adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped sink adjustment", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.attempts as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedSinkAdjustment {
        CappedSinkAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            source: self.source.clone(),
            minimum: self.minimum,
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            attempts: self.attempts,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a capped sink adjustment in status draft.
pub fn new_capped_sink_adjustment(code: &str, label: &str) -> CappedSinkAdjustment {
    CappedSinkAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        source: "".to_string(),
        minimum: 0.0,
        title: "".to_string(),
        timezone: "".to_string(),
        attempts: 0,
        cost: 0.0,
        status: CappedSinkAdjustmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_sink_adjustment("code-377", "label-151");
        assert_eq!(item.status, CappedSinkAdjustmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_capped_sink_adjustment("code-377", "label-151");
        if item.move_to(CappedSinkAdjustmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedSinkAdjustmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_sink_adjustment("code-377", "label-151");
        item.status = CappedSinkAdjustmentStatus::Closed;
        assert!(
            item.move_to(CappedSinkAdjustmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
