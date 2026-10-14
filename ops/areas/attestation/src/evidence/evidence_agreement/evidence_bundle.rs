use super::*;

use acme_kit::text;

/// Where a evidence bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EvidenceBundleStatus {
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

impl std::fmt::Display for EvidenceBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence bundle as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBundle {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub unit: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub namespace: String,
    pub revision: i64,
    pub batch_size: i64,
    pub contact_email: String,
    pub description: String,
    pub status: EvidenceBundleStatus,
}

impl EvidenceBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceBundleStatus) -> bool {
        match self.status {
            EvidenceBundleStatus::Idle => {
                match next {
                    EvidenceBundleStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceBundleStatus::Warming => {
                return next == EvidenceBundleStatus::Ready || next == EvidenceBundleStatus::Stopped;
            }
            EvidenceBundleStatus::Ready => {
                match next {
                    EvidenceBundleStatus::Draining | EvidenceBundleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceBundleStatus::Draining => {
                match next {
                    EvidenceBundleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: EvidenceBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.batch_size as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceBundle {
        EvidenceBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            unit: self.unit.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            namespace: self.namespace.clone(),
            revision: self.revision,
            batch_size: self.batch_size,
            contact_email: self.contact_email.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a evidence bundle in status idle.
pub fn new_evidence_bundle(code: &str, label: &str) -> EvidenceBundle {
    EvidenceBundle {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        unit: "".to_string(),
        title: "".to_string(),
        ttl_seconds: 0,
        namespace: "".to_string(),
        revision: 0,
        batch_size: 0,
        contact_email: "".to_string(),
        description: "".to_string(),
        status: EvidenceBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_evidence_bundle("code-399", "label-82");
        assert_eq!(item.status, EvidenceBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_evidence_bundle("code-399", "label-82");
        if item.move_to(EvidenceBundleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceBundleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_evidence_bundle("code-399", "label-82");
        item.status = EvidenceBundleStatus::Stopped;
        assert!(
            item.move_to(EvidenceBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
