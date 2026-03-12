use std::{collections::HashMap, str::FromStr};

use chrono_tz::Tz;
use croner::Cron;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub jobs: HashMap<String, Job>,
    #[serde(default = "default_timezone")]
    pub timezone: Tz,
}
fn default_timezone() -> Tz {
    Tz::from_str(&iana_time_zone::get_timezone().expect("failed to get system timezone"))
        .expect("failed parsing system timezone")
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Job {
    pub trigger: Cron,
    pub command: String,
}
impl std::fmt::Debug for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Job")
            .field("trigger", &self.trigger.pattern.to_string())
            .field("command", &self.command)
            .finish()
    }
}
