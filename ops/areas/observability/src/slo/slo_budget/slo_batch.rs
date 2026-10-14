use super::*;

use acme_kit::text;

/// Where a slo batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloBatchStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SloBatchStatus {
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

impl std::fmt::Display for SloBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SloBatch {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub summary: String,
    pub weight: f64,
    pub label: String,
    pub pinned: bool,
    pub source: String,
    pub status: SloBatchStatus,
}

impl SloBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloBatchStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloBatchStatus) -> bool {
        match self.status {
            SloBatchStatus::Draft => {
                match next {
                    SloBatchStatus::Suspended | SloBatchStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloBatchStatus::Pending => {
                return next == SloBatchStatus::Suspended;
            }
            SloBatchStatus::Active => {
                return next == SloBatchStatus::Closed;
            }
            SloBatchStatus::Suspended => {
                return next == SloBatchStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "slo batch", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.threshold * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloBatch {
        SloBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            summary: self.summary.clone(),
            weight: self.weight,
            label: self.label.clone(),
            pinned: self.pinned,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a slo batch in status draft.
pub fn new_slo_batch(code: &str, title: &str) -> SloBatch {
    SloBatch {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        summary: "".to_string(),
        weight: 0.0,
        label: "".to_string(),
        pinned: false,
        source: "".to_string(),
        status: SloBatchStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_slo_batch("code-85", "title-608");
        assert_eq!(item.status, SloBatchStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_slo_batch("code-85", "title-608");
        if item.move_to(SloBatchStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloBatchStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_slo_batch("code-85", "title-608");
        item.status = SloBatchStatus::Closed;
        assert!(item.move_to(SloBatchStatus::Draft).is_err(), "expected move to fail");
    }
}
