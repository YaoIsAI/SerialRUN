//! SerialRUN Mobile App — Desktop runner
//!
//! Runs the mobile UI on desktop for testing.
//! Connect your serial device via USB and test the mobile interface.

fn main() {
    env_logger::init();
    serialrun_mobile_app::run_desktop();
}
