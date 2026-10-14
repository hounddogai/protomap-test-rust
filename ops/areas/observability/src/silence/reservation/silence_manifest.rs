use super::*;

use acme_kit::text;

/// Where a silence manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SilenceManifestStatus {
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

impl std::fmt::Display for SilenceManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One silence manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceManifest {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub batch_size: i64,
    pub margin: f64,
    pub unit: String,
    pub summary: String,
    pub status: SilenceManifestStatus,
}

impl SilenceManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceManifestStatus) -> bool {
        match self.status {
            SilenceManifestStatus::Idle => {
                return next == SilenceManifestStatus::Warming;
            }
            SilenceManifestStatus::Warming => {
                return next == SilenceManifestStatus::Ready || next == SilenceManifestStatus::Draining;
            }
            SilenceManifestStatus::Ready => {
                match next {
                    SilenceManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceManifestStatus::Draining => {
                match next {
                    SilenceManifestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SilenceManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "silence manifest", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.margin - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceManifest {
        SilenceManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            batch_size: self.batch_size,
            margin: self.margin,
            unit: self.unit.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a silence manifest in status idle.
pub fn new_silence_manifest(code: &str, label: &str) -> SilenceManifest {
    SilenceManifest {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        batch_size: 0,
        margin: 0.0,
        unit: "".to_string(),
        summary: "".to_string(),
        status: SilenceManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_silence_manifest("code-534", "label-701");
        assert_eq!(item.status, SilenceManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_silence_manifest("code-534", "label-701");
        if item.move_to(SilenceManifestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceManifestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_silence_manifest("code-534", "label-701");
        item.status = SilenceManifestStatus::Stopped;
        assert!(
            item.move_to(SilenceManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
