use super::*;

use acme_kit::text;

/// Where a primary page baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryPageBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryPageBaselineStatus {
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

impl std::fmt::Display for PrimaryPageBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary page baseline as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPageBaseline {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub expires_at: i64,
    pub checksum: String,
    pub cluster: String,
    pub name: String,
    pub notes: String,
    pub burst: i64,
    pub status: PrimaryPageBaselineStatus,
}

impl PrimaryPageBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryPageBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryPageBaselineStatus) -> bool {
        match self.status {
            PrimaryPageBaselineStatus::Idle => {
                match next {
                    PrimaryPageBaselineStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPageBaselineStatus::Warming => {
                match next {
                    PrimaryPageBaselineStatus::Ready | PrimaryPageBaselineStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPageBaselineStatus::Ready => {
                match next {
                    PrimaryPageBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPageBaselineStatus::Draining => {
                return next == PrimaryPageBaselineStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryPageBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary page baseline {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.version as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryPageBaseline {
        PrimaryPageBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            expires_at: self.expires_at,
            checksum: self.checksum.clone(),
            cluster: self.cluster.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary page baseline in status idle.
pub fn new_primary_page_baseline(code: &str, title: &str) -> PrimaryPageBaseline {
    PrimaryPageBaseline {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        expires_at: 0,
        checksum: "".to_string(),
        cluster: "".to_string(),
        name: "".to_string(),
        notes: "".to_string(),
        burst: 0,
        status: PrimaryPageBaselineStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_page_baseline("code-920", "title-569");
        assert_eq!(item.status, PrimaryPageBaselineStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_primary_page_baseline("code-920", "title-569");
        if item.move_to(PrimaryPageBaselineStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryPageBaselineStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_page_baseline("code-920", "title-569");
        item.status = PrimaryPageBaselineStatus::Stopped;
        assert!(
            item.move_to(PrimaryPageBaselineStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
