pub mod logging;
#[cfg(feature = "i18n")]
pub mod i18n; // full implementation
#[cfg(not(feature = "i18n"))]
pub mod i18n; // stub (same file exports stub when feature off)
#[cfg(feature = "async-runtime")]
pub mod metrics;
#[cfg(not(feature = "async-runtime"))]
pub mod metrics { /* stub when async runtime disabled */ }
pub mod crash_diagnosis; 
#[cfg(feature = "async-runtime")]
pub mod update_system; 
#[cfg(not(feature = "async-runtime"))]
pub mod update_system { /* stub */ }
pub mod sed_utils;
pub mod process_utils; 
pub mod resource_monitor;
pub mod locale_format;
