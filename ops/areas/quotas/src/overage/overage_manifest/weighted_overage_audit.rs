use super::*;

use acme_kit::text;

/// Where a weighted overage audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedOverageAuditStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedOverageAuditStatus {
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

impl std::fmt::Display for WeightedOverageAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted overage audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageAudit {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub audience: String,
    pub principal: String,
    pub quantity: i64,
    pub status: WeightedOverageAuditStatus,
}

impl WeightedOverageAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedOverageAuditStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedOverageAuditStatus) -> bool {
        match self.status {
            WeightedOverageAuditStatus::Planned => {
                match next {
                    WeightedOverageAuditStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedOverageAuditStatus::Scheduled => {
                match next {
                    WeightedOverageAuditStatus::InProgress | WeightedOverageAuditStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedOverageAuditStatus::InProgress => {
                return next == WeightedOverageAuditStatus::Cancelled;
            }
            WeightedOverageAuditStatus::Completed => {
                match next {
                    WeightedOverageAuditStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: WeightedOverageAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted overage audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> WeightedOverageAudit {
        WeightedOverageAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            audience: self.audience.clone(),
            principal: self.principal.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted overage audit in status planned.
pub fn new_weighted_overage_audit(code: &str, title: &str) -> WeightedOverageAudit {
    WeightedOverageAudit {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        audience: "".to_string(),
        principal: "".to_string(),
        quantity: 0,
        status: WeightedOverageAuditStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_overage_audit("code-160", "title-887");
        assert_eq!(item.status, WeightedOverageAuditStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_weighted_overage_audit("code-160", "title-887");
        if item.move_to(WeightedOverageAuditStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedOverageAuditStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_overage_audit("code-160", "title-887");
        item.status = WeightedOverageAuditStatus::Cancelled;
        assert!(
            item.move_to(WeightedOverageAuditStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
