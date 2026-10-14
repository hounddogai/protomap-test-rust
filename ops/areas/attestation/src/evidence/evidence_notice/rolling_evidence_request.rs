use super::*;

use acme_kit::text;

/// Where a rolling evidence request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingEvidenceRequestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingEvidenceRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingEvidenceRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling evidence request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEvidenceRequest {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub attempts: i64,
    pub audience: String,
    pub tags: Vec<String>,
    pub locale: String,
    pub status: RollingEvidenceRequestStatus,
}

impl RollingEvidenceRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingEvidenceRequestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingEvidenceRequestStatus) -> bool {
        match self.status {
            RollingEvidenceRequestStatus::Created => {
                return next == RollingEvidenceRequestStatus::Published
                    || next == RollingEvidenceRequestStatus::Retired;
            }
            RollingEvidenceRequestStatus::Validated => {
                match next {
                    RollingEvidenceRequestStatus::Published | RollingEvidenceRequestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingEvidenceRequestStatus::Published => {
                return next == RollingEvidenceRequestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingEvidenceRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling evidence request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> RollingEvidenceRequest {
        RollingEvidenceRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            attempts: self.attempts,
            audience: self.audience.clone(),
            tags: self.tags.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rolling evidence request in status created.
pub fn new_rolling_evidence_request(code: &str, title: &str) -> RollingEvidenceRequest {
    RollingEvidenceRequest {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        attempts: 0,
        audience: "".to_string(),
        tags: Vec::new(),
        locale: "".to_string(),
        status: RollingEvidenceRequestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rolling_evidence_request("code-422", "title-79");
        assert_eq!(item.status, RollingEvidenceRequestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_rolling_evidence_request("code-422", "title-79");
        if item.move_to(RollingEvidenceRequestStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingEvidenceRequestStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rolling_evidence_request("code-422", "title-79");
        item.status = RollingEvidenceRequestStatus::Retired;
        assert!(
            item.move_to(RollingEvidenceRequestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
