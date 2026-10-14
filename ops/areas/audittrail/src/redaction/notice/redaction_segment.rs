use super::*;

use acme_kit::text;

/// Where a redaction segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionSegmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RedactionSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RedactionSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSegment {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub minimum: f64,
    pub updated_at: i64,
    pub timezone: String,
    pub cipher: String,
    pub margin: f64,
    pub unit: String,
    pub ttl_seconds: i64,
    pub visible: bool,
    pub status: RedactionSegmentStatus,
}

impl RedactionSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionSegmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionSegmentStatus) -> bool {
        match self.status {
            RedactionSegmentStatus::Requested => {
                return next == RedactionSegmentStatus::Granted || next == RedactionSegmentStatus::Revoked;
            }
            RedactionSegmentStatus::Granted => {
                match next {
                    RedactionSegmentStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionSegmentStatus::Revoked => {
                match next {
                    RedactionSegmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: RedactionSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "redaction segment", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.margin - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionSegment {
        RedactionSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            minimum: self.minimum,
            updated_at: self.updated_at,
            timezone: self.timezone.clone(),
            cipher: self.cipher.clone(),
            margin: self.margin,
            unit: self.unit.clone(),
            ttl_seconds: self.ttl_seconds,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a redaction segment in status requested.
pub fn new_redaction_segment(code: &str, label: &str) -> RedactionSegment {
    RedactionSegment {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        minimum: 0.0,
        updated_at: 0,
        timezone: "".to_string(),
        cipher: "".to_string(),
        margin: 0.0,
        unit: "".to_string(),
        ttl_seconds: 0,
        visible: false,
        status: RedactionSegmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_redaction_segment("code-405", "label-604");
        assert_eq!(item.status, RedactionSegmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_redaction_segment("code-405", "label-604");
        if item.move_to(RedactionSegmentStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionSegmentStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_redaction_segment("code-405", "label-604");
        item.status = RedactionSegmentStatus::Expired;
        assert!(
            item.move_to(RedactionSegmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
