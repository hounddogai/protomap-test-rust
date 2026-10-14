use super::*;

use acme_kit::text;

/// Where a limit variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LimitVarianceStatus {
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

impl std::fmt::Display for LimitVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit variance as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitVariance {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub reviewer: String,
    pub country: String,
    pub score: f64,
    pub phone: String,
    pub expires_at: i64,
    pub checksum: String,
    pub tags: Vec<String>,
    pub contact_email: String,
    pub status: LimitVarianceStatus,
}

impl LimitVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitVarianceStatus) -> bool {
        match self.status {
            LimitVarianceStatus::Idle => {
                return next == LimitVarianceStatus::Ready || next == LimitVarianceStatus::Draining;
            }
            LimitVarianceStatus::Warming => {
                match next {
                    LimitVarianceStatus::Draining | LimitVarianceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitVarianceStatus::Ready => {
                return next == LimitVarianceStatus::Stopped;
            }
            LimitVarianceStatus::Draining => {
                return next == LimitVarianceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.batch_size as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitVariance {
        LimitVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            reviewer: self.reviewer.clone(),
            country: self.country.clone(),
            score: self.score,
            phone: self.phone.clone(),
            expires_at: self.expires_at,
            checksum: self.checksum.clone(),
            tags: self.tags.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a limit variance in status idle.
pub fn new_limit_variance(code: &str, title: &str) -> LimitVariance {
    LimitVariance {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        reviewer: "".to_string(),
        country: "".to_string(),
        score: 0.0,
        phone: "".to_string(),
        expires_at: 0,
        checksum: "".to_string(),
        tags: Vec::new(),
        contact_email: "".to_string(),
        status: LimitVarianceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_limit_variance("code-837", "title-876");
        assert_eq!(item.status, LimitVarianceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_limit_variance("code-837", "title-876");
        if item.move_to(LimitVarianceStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitVarianceStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_limit_variance("code-837", "title-876");
        item.status = LimitVarianceStatus::Stopped;
        assert!(
            item.move_to(LimitVarianceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
