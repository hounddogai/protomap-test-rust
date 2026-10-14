use super::*;

use acme_kit::text;

/// Where a slo preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SloPreferenceStatus {
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

impl std::fmt::Display for SloPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo preference as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloPreference {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub cluster: String,
    pub threshold: f64,
    pub grace_days: i64,
    pub version: i64,
    pub description: String,
    pub reviewer: String,
    pub status: SloPreferenceStatus,
}

impl SloPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloPreferenceStatus) -> bool {
        match self.status {
            SloPreferenceStatus::Idle => {
                return next == SloPreferenceStatus::Draining || next == SloPreferenceStatus::Stopped;
            }
            SloPreferenceStatus::Warming => {
                return next == SloPreferenceStatus::Ready || next == SloPreferenceStatus::Stopped;
            }
            SloPreferenceStatus::Ready => {
                match next {
                    SloPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloPreferenceStatus::Draining => {
                match next {
                    SloPreferenceStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SloPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "slo preference", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.grace_days as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloPreference {
        SloPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            cluster: self.cluster.clone(),
            threshold: self.threshold,
            grace_days: self.grace_days,
            version: self.version,
            description: self.description.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a slo preference in status idle.
pub fn new_slo_preference(code: &str, label: &str) -> SloPreference {
    SloPreference {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        cluster: "".to_string(),
        threshold: 0.0,
        grace_days: 0,
        version: 0,
        description: "".to_string(),
        reviewer: "".to_string(),
        status: SloPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_slo_preference("code-873", "label-178");
        assert_eq!(item.status, SloPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_slo_preference("code-873", "label-178");
        if item.move_to(SloPreferenceStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloPreferenceStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_slo_preference("code-873", "label-178");
        item.status = SloPreferenceStatus::Stopped;
        assert!(
            item.move_to(SloPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
