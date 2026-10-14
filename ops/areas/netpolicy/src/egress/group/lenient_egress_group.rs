use super::*;

use acme_kit::text;

/// Where a lenient egress group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientEgressGroupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientEgressGroupStatus {
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

impl std::fmt::Display for LenientEgressGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient egress group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressGroup {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub rotation_days: i64,
    pub quantity: i64,
    pub issuer: String,
    pub status: LenientEgressGroupStatus,
}

impl LenientEgressGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientEgressGroupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientEgressGroupStatus) -> bool {
        match self.status {
            LenientEgressGroupStatus::New => {
                return next == LenientEgressGroupStatus::Queued || next == LenientEgressGroupStatus::Failed;
            }
            LenientEgressGroupStatus::Queued => {
                match next {
                    LenientEgressGroupStatus::Running | LenientEgressGroupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEgressGroupStatus::Running => {
                return next == LenientEgressGroupStatus::Done;
            }
            LenientEgressGroupStatus::Done => {
                match next {
                    LenientEgressGroupStatus::Failed => {
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

    pub fn move_to(&mut self, next: LenientEgressGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient egress group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.quantity as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientEgressGroup {
        LenientEgressGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            rotation_days: self.rotation_days,
            quantity: self.quantity,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient egress group in status new.
pub fn new_lenient_egress_group(code: &str, label: &str) -> LenientEgressGroup {
    LenientEgressGroup {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        rotation_days: 0,
        quantity: 0,
        issuer: "".to_string(),
        status: LenientEgressGroupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_egress_group("code-719", "label-260");
        assert_eq!(item.status, LenientEgressGroupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_lenient_egress_group("code-719", "label-260");
        if item.move_to(LenientEgressGroupStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientEgressGroupStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_egress_group("code-719", "label-260");
        item.status = LenientEgressGroupStatus::Failed;
        assert!(
            item.move_to(LenientEgressGroupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
