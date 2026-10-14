use super::*;

use acme_kit::text;

/// Where a manual dns record adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDnsRecordAdjustmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualDnsRecordAdjustmentStatus {
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

impl std::fmt::Display for ManualDnsRecordAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual dns record adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordAdjustment {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub external_ref: String,
    pub version: i64,
    pub scope: String,
    pub status: ManualDnsRecordAdjustmentStatus,
}

impl ManualDnsRecordAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDnsRecordAdjustmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDnsRecordAdjustmentStatus) -> bool {
        match self.status {
            ManualDnsRecordAdjustmentStatus::Idle => {
                return next == ManualDnsRecordAdjustmentStatus::Warming;
            }
            ManualDnsRecordAdjustmentStatus::Warming => {
                match next {
                    ManualDnsRecordAdjustmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDnsRecordAdjustmentStatus::Ready => {
                match next {
                    ManualDnsRecordAdjustmentStatus::Draining | ManualDnsRecordAdjustmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDnsRecordAdjustmentStatus::Draining => {
                match next {
                    ManualDnsRecordAdjustmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ManualDnsRecordAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual dns record adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        (self.score * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualDnsRecordAdjustment {
        ManualDnsRecordAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            external_ref: self.external_ref.clone(),
            version: self.version,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a manual dns record adjustment in status idle.
pub fn new_manual_dns_record_adjustment(code: &str, label: &str) -> ManualDnsRecordAdjustment {
    ManualDnsRecordAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        external_ref: "".to_string(),
        version: 0,
        scope: "".to_string(),
        status: ManualDnsRecordAdjustmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_dns_record_adjustment("code-11", "label-178");
        assert_eq!(item.status, ManualDnsRecordAdjustmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_manual_dns_record_adjustment("code-11", "label-178");
        if item.move_to(ManualDnsRecordAdjustmentStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualDnsRecordAdjustmentStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_dns_record_adjustment("code-11", "label-178");
        item.status = ManualDnsRecordAdjustmentStatus::Stopped;
        assert!(
            item.move_to(ManualDnsRecordAdjustmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
