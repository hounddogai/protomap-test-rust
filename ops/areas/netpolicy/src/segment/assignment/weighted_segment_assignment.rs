use super::*;

use acme_kit::text;

/// Where a weighted segment assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedSegmentAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedSegmentAssignmentStatus {
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

impl std::fmt::Display for WeightedSegmentAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted segment assignment as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentAssignment {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub currency: String,
    pub budget: f64,
    pub region: String,
    pub unit: String,
    pub phone: String,
    pub amount: f64,
    pub status: WeightedSegmentAssignmentStatus,
}

impl WeightedSegmentAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedSegmentAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedSegmentAssignmentStatus) -> bool {
        match self.status {
            WeightedSegmentAssignmentStatus::Draft => {
                match next {
                    WeightedSegmentAssignmentStatus::Active | WeightedSegmentAssignmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedSegmentAssignmentStatus::Pending => {
                match next {
                    WeightedSegmentAssignmentStatus::Suspended | WeightedSegmentAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedSegmentAssignmentStatus::Active => {
                return next == WeightedSegmentAssignmentStatus::Suspended
                    || next == WeightedSegmentAssignmentStatus::Closed;
            }
            WeightedSegmentAssignmentStatus::Suspended => {
                match next {
                    WeightedSegmentAssignmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: WeightedSegmentAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted segment assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "weighted segment assignment", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.weight + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedSegmentAssignment {
        WeightedSegmentAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            currency: self.currency.clone(),
            budget: self.budget,
            region: self.region.clone(),
            unit: self.unit.clone(),
            phone: self.phone.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a weighted segment assignment in status draft.
pub fn new_weighted_segment_assignment(code: &str, label: &str) -> WeightedSegmentAssignment {
    WeightedSegmentAssignment {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        currency: "".to_string(),
        budget: 0.0,
        region: "".to_string(),
        unit: "".to_string(),
        phone: "".to_string(),
        amount: 0.0,
        status: WeightedSegmentAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_weighted_segment_assignment("code-297", "label-570");
        assert_eq!(item.status, WeightedSegmentAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_weighted_segment_assignment("code-297", "label-570");
        if item.move_to(WeightedSegmentAssignmentStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedSegmentAssignmentStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_weighted_segment_assignment("code-297", "label-570");
        item.status = WeightedSegmentAssignmentStatus::Closed;
        assert!(
            item.move_to(WeightedSegmentAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
