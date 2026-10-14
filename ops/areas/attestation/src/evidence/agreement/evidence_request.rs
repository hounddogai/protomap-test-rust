use super::*;

use acme_kit::text;

/// Where a evidence request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceRequestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EvidenceRequestStatus {
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

impl std::fmt::Display for EvidenceRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRequest {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub discount: f64,
    pub tags: Vec<String>,
    pub version: i64,
    pub expires_at: i64,
    pub reviewer: String,
    pub status: EvidenceRequestStatus,
}

impl EvidenceRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceRequestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceRequestStatus) -> bool {
        match self.status {
            EvidenceRequestStatus::Idle => {
                match next {
                    EvidenceRequestStatus::Draining | EvidenceRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceRequestStatus::Warming => {
                return next == EvidenceRequestStatus::Ready;
            }
            EvidenceRequestStatus::Ready => {
                return next == EvidenceRequestStatus::Stopped;
            }
            EvidenceRequestStatus::Draining => {
                match next {
                    EvidenceRequestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: EvidenceRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        ((self.maximum + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceRequest {
        EvidenceRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            discount: self.discount,
            tags: self.tags.clone(),
            version: self.version,
            expires_at: self.expires_at,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a evidence request in status idle.
pub fn new_evidence_request(code: &str, label: &str) -> EvidenceRequest {
    EvidenceRequest {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        discount: 0.0,
        tags: Vec::new(),
        version: 0,
        expires_at: 0,
        reviewer: "".to_string(),
        status: EvidenceRequestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_evidence_request("code-869", "label-855");
        assert_eq!(item.status, EvidenceRequestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_evidence_request("code-869", "label-855");
        if item.move_to(EvidenceRequestStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceRequestStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_evidence_request("code-869", "label-855");
        item.status = EvidenceRequestStatus::Stopped;
        assert!(
            item.move_to(EvidenceRequestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
