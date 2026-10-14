use super::*;

use acme_kit::text;

/// Where a lenient page variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPageVarianceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientPageVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientPageVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient page variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageVariance {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub key_id: String,
    pub starts_at: i64,
    pub status: LenientPageVarianceStatus,
}

impl LenientPageVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPageVarianceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPageVarianceStatus) -> bool {
        match self.status {
            LenientPageVarianceStatus::Requested => {
                return next == LenientPageVarianceStatus::Expired;
            }
            LenientPageVarianceStatus::Granted => {
                match next {
                    LenientPageVarianceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPageVarianceStatus::Revoked => {
                return next == LenientPageVarianceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientPageVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient page variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient page variance", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> LenientPageVariance {
        LenientPageVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            key_id: self.key_id.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a lenient page variance in status requested.
pub fn new_lenient_page_variance(code: &str, label: &str) -> LenientPageVariance {
    LenientPageVariance {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        key_id: "".to_string(),
        starts_at: 0,
        status: LenientPageVarianceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_page_variance("code-4", "label-189");
        assert_eq!(item.status, LenientPageVarianceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_lenient_page_variance("code-4", "label-189");
        if item.move_to(LenientPageVarianceStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPageVarianceStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_page_variance("code-4", "label-189");
        item.status = LenientPageVarianceStatus::Expired;
        assert!(
            item.move_to(LenientPageVarianceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
