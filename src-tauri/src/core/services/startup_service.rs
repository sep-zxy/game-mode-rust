use crate::core::domain::error::AppResult;
use crate::infra::windows::startup;

#[derive(Default)]
pub struct StartupService;

impl StartupService {
    pub fn new() -> Self {
        Self
    }

    pub fn is_enabled(&self) -> AppResult<bool> {
        startup::is_enabled()
    }

    pub fn set_enabled(&self, enabled: bool) -> AppResult<()> {
        startup::set_enabled(enabled)
    }
}
