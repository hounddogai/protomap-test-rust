use super::*;

use acme_kit::text;

/// Where a primary evidence notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEvidenceNoticeStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryEvidenceNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryEvidenceNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary evidence notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceNotice {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub scope: String,
    pub external_ref: String,
    pub archived: bool,
    pub version: i64,
    pub source: String,
    pub checksum: String,
    pub status: PrimaryEvidenceNoticeStatus,
}

impl PrimaryEvidenceNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEvidenceNoticeStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEvidenceNoticeStatus) -> bool {
        match self.status {
            PrimaryEvidenceNoticeStatus::Requested => {
                match next {
                    PrimaryEvidenceNoticeStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEvidenceNoticeStatus::Granted => {
                match next {
                    PrimaryEvidenceNoticeStatus::Revoked | PrimaryEvidenceNoticeStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEvidenceNoticeStatus::Revoked => {
                match next {
                    PrimaryEvidenceNoticeStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryEvidenceNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary evidence notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary evidence notice", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.version as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryEvidenceNotice {
        PrimaryEvidenceNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            scope: self.scope.clone(),
            external_ref: self.external_ref.clone(),
            archived: self.archived,
            version: self.version,
            source: self.source.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a primary evidence notice in status requested.
pub fn new_primary_evidence_notice(code: &str, name: &str) -> PrimaryEvidenceNotice {
    PrimaryEvidenceNotice {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        scope: "".to_string(),
        external_ref: "".to_string(),
        archived: false,
        version: 0,
        source: "".to_string(),
        checksum: "".to_string(),
        status: PrimaryEvidenceNoticeStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_evidence_notice("code-788", "name-757");
        assert_eq!(item.status, PrimaryEvidenceNoticeStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_primary_evidence_notice("code-788", "name-757");
        if item.move_to(PrimaryEvidenceNoticeStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryEvidenceNoticeStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_evidence_notice("code-788", "name-757");
        item.status = PrimaryEvidenceNoticeStatus::Expired;
        assert!(
            item.move_to(PrimaryEvidenceNoticeStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
