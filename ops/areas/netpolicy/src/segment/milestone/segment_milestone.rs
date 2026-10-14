use super::*;

use acme_kit::text;

/// Where a segment milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SegmentMilestoneStatus {
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

impl std::fmt::Display for SegmentMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMilestone {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub visible: bool,
    pub ratio: f64,
    pub revision: i64,
    pub issuer: String,
    pub limit: i64,
    pub updated_at: i64,
    pub status: SegmentMilestoneStatus,
}

impl SegmentMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentMilestoneStatus) -> bool {
        match self.status {
            SegmentMilestoneStatus::Draft => {
                match next {
                    SegmentMilestoneStatus::Suspended | SegmentMilestoneStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentMilestoneStatus::Pending => {
                return next == SegmentMilestoneStatus::Active || next == SegmentMilestoneStatus::Closed;
            }
            SegmentMilestoneStatus::Active => {
                match next {
                    SegmentMilestoneStatus::Suspended | SegmentMilestoneStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentMilestoneStatus::Suspended => {
                return next == SegmentMilestoneStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.limit as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentMilestone {
        SegmentMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            visible: self.visible,
            ratio: self.ratio,
            revision: self.revision,
            issuer: self.issuer.clone(),
            limit: self.limit,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a segment milestone in status draft.
pub fn new_segment_milestone(code: &str, label: &str) -> SegmentMilestone {
    SegmentMilestone {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        visible: false,
        ratio: 0.0,
        revision: 0,
        issuer: "".to_string(),
        limit: 0,
        updated_at: 0,
        status: SegmentMilestoneStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_segment_milestone("code-383", "label-556");
        assert_eq!(item.status, SegmentMilestoneStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_segment_milestone("code-383", "label-556");
        if item.move_to(SegmentMilestoneStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentMilestoneStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_segment_milestone("code-383", "label-556");
        item.status = SegmentMilestoneStatus::Closed;
        assert!(
            item.move_to(SegmentMilestoneStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
