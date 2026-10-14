use super::*;

use acme_kit::text;

/// Where a segment estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentEstimateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SegmentEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for SegmentEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEstimate {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub name: String,
    pub contact_email: String,
    pub amount: f64,
    pub currency: String,
    pub region: String,
    pub status: SegmentEstimateStatus,
}

impl SegmentEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentEstimateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentEstimateStatus) -> bool {
        match self.status {
            SegmentEstimateStatus::New => {
                match next {
                    SegmentEstimateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentEstimateStatus::Queued => {
                match next {
                    SegmentEstimateStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentEstimateStatus::Running => {
                match next {
                    SegmentEstimateStatus::Done | SegmentEstimateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentEstimateStatus::Done => {
                return next == SegmentEstimateStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> SegmentEstimate {
        SegmentEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            amount: self.amount,
            currency: self.currency.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a segment estimate in status new.
pub fn new_segment_estimate(code: &str, label: &str) -> SegmentEstimate {
    SegmentEstimate {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        name: "".to_string(),
        contact_email: "".to_string(),
        amount: 0.0,
        currency: "".to_string(),
        region: "".to_string(),
        status: SegmentEstimateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_segment_estimate("code-890", "label-449");
        assert_eq!(item.status, SegmentEstimateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_segment_estimate("code-890", "label-449");
        if item.move_to(SegmentEstimateStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentEstimateStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_segment_estimate("code-890", "label-449");
        item.status = SegmentEstimateStatus::Failed;
        assert!(
            item.move_to(SegmentEstimateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
