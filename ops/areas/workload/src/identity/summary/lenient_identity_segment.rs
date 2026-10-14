use super::*;

use acme_kit::text;

/// Where a lenient identity segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientIdentitySegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientIdentitySegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for LenientIdentitySegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient identity segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientIdentitySegment {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub limit: i64,
    pub cluster: String,
    pub phone: String,
    pub cipher: String,
    pub host: String,
    pub updated_at: i64,
    pub status: LenientIdentitySegmentStatus,
}

impl LenientIdentitySegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientIdentitySegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientIdentitySegmentStatus) -> bool {
        match self.status {
            LenientIdentitySegmentStatus::New => {
                return next == LenientIdentitySegmentStatus::Failed;
            }
            LenientIdentitySegmentStatus::Queued => {
                return next == LenientIdentitySegmentStatus::Running || next == LenientIdentitySegmentStatus::Failed;
            }
            LenientIdentitySegmentStatus::Running => {
                match next {
                    LenientIdentitySegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientIdentitySegmentStatus::Done => {
                match next {
                    LenientIdentitySegmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: LenientIdentitySegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient identity segment {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.updated_at as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientIdentitySegment {
        LenientIdentitySegment {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            limit: self.limit,
            cluster: self.cluster.clone(),
            phone: self.phone.clone(),
            cipher: self.cipher.clone(),
            host: self.host.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a lenient identity segment in status new.
pub fn new_lenient_identity_segment(code: &str, name: &str) -> LenientIdentitySegment {
    LenientIdentitySegment {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        limit: 0,
        cluster: "".to_string(),
        phone: "".to_string(),
        cipher: "".to_string(),
        host: "".to_string(),
        updated_at: 0,
        status: LenientIdentitySegmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_identity_segment("code-870", "name-744");
        assert_eq!(item.status, LenientIdentitySegmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_lenient_identity_segment("code-870", "name-744");
        if item.move_to(LenientIdentitySegmentStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientIdentitySegmentStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_identity_segment("code-870", "name-744");
        item.status = LenientIdentitySegmentStatus::Failed;
        assert!(
            item.move_to(LenientIdentitySegmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
