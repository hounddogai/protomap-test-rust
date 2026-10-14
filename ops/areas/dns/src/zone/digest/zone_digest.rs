use super::*;

use acme_kit::text;

/// Where a zone digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ZoneDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ZoneDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a zone digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDigest {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub fingerprint: String,
    pub ends_at: i64,
    pub summary: String,
    pub active: bool,
    pub reviewer: String,
    pub version: i64,
    pub channel: String,
    pub status: ZoneDigestStatus,
}

impl ZoneDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneDigestStatus) -> bool {
        match self.status {
            ZoneDigestStatus::Proposed => {
                return next == ZoneDigestStatus::Rejected;
            }
            ZoneDigestStatus::Approved => {
                match next {
                    ZoneDigestStatus::Rejected | ZoneDigestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneDigestStatus::Rejected => {
                return next == ZoneDigestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.minimum + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneDigest {
        ZoneDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            fingerprint: self.fingerprint.clone(),
            ends_at: self.ends_at,
            summary: self.summary.clone(),
            active: self.active,
            reviewer: self.reviewer.clone(),
            version: self.version,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a zone digest in status proposed.
pub fn new_zone_digest(code: &str, title: &str) -> ZoneDigest {
    ZoneDigest {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        fingerprint: "".to_string(),
        ends_at: 0,
        summary: "".to_string(),
        active: false,
        reviewer: "".to_string(),
        version: 0,
        channel: "".to_string(),
        status: ZoneDigestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_zone_digest("code-118", "title-245");
        assert_eq!(item.status, ZoneDigestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_zone_digest("code-118", "title-245");
        if item.move_to(ZoneDigestStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneDigestStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_zone_digest("code-118", "title-245");
        item.status = ZoneDigestStatus::Withdrawn;
        assert!(
            item.move_to(ZoneDigestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
