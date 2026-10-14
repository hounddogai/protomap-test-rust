use super::*;

use acme_kit::text;

/// Where a lenient suspension profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSuspensionProfileStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientSuspensionProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for LenientSuspensionProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient suspension profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionProfile {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub minimum: f64,
    pub key_id: String,
    pub sequence: i64,
    pub batch_size: i64,
    pub approved: bool,
    pub threshold: f64,
    pub status: LenientSuspensionProfileStatus,
}

impl LenientSuspensionProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSuspensionProfileStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSuspensionProfileStatus) -> bool {
        match self.status {
            LenientSuspensionProfileStatus::Idle => {
                return next == LenientSuspensionProfileStatus::Ready
                    || next == LenientSuspensionProfileStatus::Draining;
            }
            LenientSuspensionProfileStatus::Warming => {
                return next == LenientSuspensionProfileStatus::Ready
                    || next == LenientSuspensionProfileStatus::Draining;
            }
            LenientSuspensionProfileStatus::Ready => {
                match next {
                    LenientSuspensionProfileStatus::Draining | LenientSuspensionProfileStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSuspensionProfileStatus::Draining => {
                match next {
                    LenientSuspensionProfileStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LenientSuspensionProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient suspension profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.margin - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSuspensionProfile {
        LenientSuspensionProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            minimum: self.minimum,
            key_id: self.key_id.clone(),
            sequence: self.sequence,
            batch_size: self.batch_size,
            approved: self.approved,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient suspension profile in status idle.
pub fn new_lenient_suspension_profile(code: &str, label: &str) -> LenientSuspensionProfile {
    LenientSuspensionProfile {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        minimum: 0.0,
        key_id: "".to_string(),
        sequence: 0,
        batch_size: 0,
        approved: false,
        threshold: 0.0,
        status: LenientSuspensionProfileStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_suspension_profile("code-984", "label-398");
        assert_eq!(item.status, LenientSuspensionProfileStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_lenient_suspension_profile("code-984", "label-398");
        if item.move_to(LenientSuspensionProfileStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSuspensionProfileStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_suspension_profile("code-984", "label-398");
        item.status = LenientSuspensionProfileStatus::Stopped;
        assert!(
            item.move_to(LenientSuspensionProfileStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
