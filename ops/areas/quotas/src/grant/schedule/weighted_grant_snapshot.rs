use super::*;

use acme_kit::text;

/// Where a weighted grant snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedGrantSnapshotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedGrantSnapshotStatus {
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

impl std::fmt::Display for WeightedGrantSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted grant snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantSnapshot {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub weight: f64,
    pub name: String,
    pub cipher: String,
    pub segment: String,
    pub key_id: String,
    pub quantity: i64,
    pub scope: String,
    pub status: WeightedGrantSnapshotStatus,
}

impl WeightedGrantSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedGrantSnapshotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedGrantSnapshotStatus) -> bool {
        match self.status {
            WeightedGrantSnapshotStatus::Draft => {
                return next == WeightedGrantSnapshotStatus::Suspended || next == WeightedGrantSnapshotStatus::Closed;
            }
            WeightedGrantSnapshotStatus::Pending => {
                return next == WeightedGrantSnapshotStatus::Suspended;
            }
            WeightedGrantSnapshotStatus::Active => {
                return next == WeightedGrantSnapshotStatus::Suspended || next == WeightedGrantSnapshotStatus::Closed;
            }
            WeightedGrantSnapshotStatus::Suspended => {
                match next {
                    WeightedGrantSnapshotStatus::Closed => {
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

    pub fn move_to(&mut self, next: WeightedGrantSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted grant snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        (self.cost * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedGrantSnapshot {
        WeightedGrantSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            weight: self.weight,
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            segment: self.segment.clone(),
            key_id: self.key_id.clone(),
            quantity: self.quantity,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a weighted grant snapshot in status draft.
pub fn new_weighted_grant_snapshot(code: &str, label: &str) -> WeightedGrantSnapshot {
    WeightedGrantSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        weight: 0.0,
        name: "".to_string(),
        cipher: "".to_string(),
        segment: "".to_string(),
        key_id: "".to_string(),
        quantity: 0,
        scope: "".to_string(),
        status: WeightedGrantSnapshotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_weighted_grant_snapshot("code-218", "label-705");
        assert_eq!(item.status, WeightedGrantSnapshotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_weighted_grant_snapshot("code-218", "label-705");
        if item.move_to(WeightedGrantSnapshotStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedGrantSnapshotStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_weighted_grant_snapshot("code-218", "label-705");
        item.status = WeightedGrantSnapshotStatus::Closed;
        assert!(
            item.move_to(WeightedGrantSnapshotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
