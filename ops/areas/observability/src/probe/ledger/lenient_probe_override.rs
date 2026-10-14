use super::*;

use acme_kit::text;

/// Where a lenient probe override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientProbeOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientProbeOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LenientProbeOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient probe override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeOverride {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub currency: String,
    pub cluster: String,
    pub capacity: i64,
    pub status: LenientProbeOverrideStatus,
}

impl LenientProbeOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientProbeOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientProbeOverrideStatus) -> bool {
        match self.status {
            LenientProbeOverrideStatus::Open => {
                return next == LenientProbeOverrideStatus::Reopened;
            }
            LenientProbeOverrideStatus::InReview => {
                return next == LenientProbeOverrideStatus::Archived;
            }
            LenientProbeOverrideStatus::Resolved => {
                return next == LenientProbeOverrideStatus::Reopened || next == LenientProbeOverrideStatus::Archived;
            }
            LenientProbeOverrideStatus::Reopened => {
                match next {
                    LenientProbeOverrideStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientProbeOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient probe override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient probe override", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> LenientProbeOverride {
        LenientProbeOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            currency: self.currency.clone(),
            cluster: self.cluster.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a lenient probe override in status open.
pub fn new_lenient_probe_override(code: &str, label: &str) -> LenientProbeOverride {
    LenientProbeOverride {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        currency: "".to_string(),
        cluster: "".to_string(),
        capacity: 0,
        status: LenientProbeOverrideStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_probe_override("code-765", "label-102");
        assert_eq!(item.status, LenientProbeOverrideStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_lenient_probe_override("code-765", "label-102");
        if item.move_to(LenientProbeOverrideStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientProbeOverrideStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_probe_override("code-765", "label-102");
        item.status = LenientProbeOverrideStatus::Archived;
        assert!(
            item.move_to(LenientProbeOverrideStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
