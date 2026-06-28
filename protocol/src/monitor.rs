// SPDX-License-Identifier: MIT
use crate::inject::KeyEvent;

pub trait KeyMonitor {
    fn grab(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn ungrab(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn read_key(&self) -> Result<KeyEvent, Box<dyn std::error::Error>>;
    fn is_active(&self) -> bool;
}
