use super::*;

use acme_kit::text;

/// Where a batched zone plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedZonePlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedZonePlanStatus {
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

impl std::fmt::Display for BatchedZonePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched zone plan as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZonePlan {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub margin: f64,
    pub category: String,
    pub timezone: String,
    pub key_id: String,
    pub country: String,
    pub status: BatchedZonePlanStatus,
}

impl BatchedZonePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedZonePlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedZonePlanStatus) -> bool {
        match self.status {
            BatchedZonePlanStatus::Draft => {
                return next == BatchedZonePlanStatus::Pending || next == BatchedZonePlanStatus::Active;
            }
            BatchedZonePlanStatus::Pending => {
                match next {
                    BatchedZonePlanStatus::Active | BatchedZonePlanStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedZonePlanStatus::Active => {
                match next {
                    BatchedZonePlanStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedZonePlanStatus::Suspended => {
                match next {
                    BatchedZonePlanStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedZonePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched zone plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.margin + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedZonePlan {
        BatchedZonePlan {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            margin: self.margin,
            category: self.category.clone(),
            timezone: self.timezone.clone(),
            key_id: self.key_id.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched zone plan in status draft.
pub fn new_batched_zone_plan(code: &str, label: &str) -> BatchedZonePlan {
    BatchedZonePlan {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        margin: 0.0,
        category: "".to_string(),
        timezone: "".to_string(),
        key_id: "".to_string(),
        country: "".to_string(),
        status: BatchedZonePlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_zone_plan("code-202", "label-142");
        assert_eq!(item.status, BatchedZonePlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_batched_zone_plan("code-202", "label-142");
        if item.move_to(BatchedZonePlanStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedZonePlanStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_zone_plan("code-202", "label-142");
        item.status = BatchedZonePlanStatus::Closed;
        assert!(
            item.move_to(BatchedZonePlanStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
