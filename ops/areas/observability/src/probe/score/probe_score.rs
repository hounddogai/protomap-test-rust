use super::*;

use acme_kit::text;

/// Where a probe score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ProbeScoreStatus {
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

impl std::fmt::Display for ProbeScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe score as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeScore {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub score: f64,
    pub limit: i64,
    pub status: ProbeScoreStatus,
}

impl ProbeScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeScoreStatus) -> bool {
        match self.status {
            ProbeScoreStatus::Idle => {
                match next {
                    ProbeScoreStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeScoreStatus::Warming => {
                return next == ProbeScoreStatus::Stopped;
            }
            ProbeScoreStatus::Ready => {
                return next == ProbeScoreStatus::Draining || next == ProbeScoreStatus::Stopped;
            }
            ProbeScoreStatus::Draining => {
                match next {
                    ProbeScoreStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ProbeScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.sequence as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeScore {
        ProbeScore {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            score: self.score,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a probe score in status idle.
pub fn new_probe_score(code: &str, title: &str) -> ProbeScore {
    ProbeScore {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        score: 0.0,
        limit: 0,
        status: ProbeScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_probe_score("code-730", "title-968");
        assert_eq!(item.status, ProbeScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_probe_score("code-730", "title-968");
        if item.move_to(ProbeScoreStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeScoreStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_probe_score("code-730", "title-968");
        item.status = ProbeScoreStatus::Stopped;
        assert!(item.move_to(ProbeScoreStatus::Idle).is_err(), "expected move to fail");
    }
}
