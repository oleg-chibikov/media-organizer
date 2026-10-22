//! One location fix from CoreLocation on macOS.

use std::{
    thread,
    time::{Duration, Instant},
};

use objc2_core_location::{kCLLocationAccuracyKilometer, CLAuthorizationStatus, CLLocationManager};
use objc2_foundation::{NSDate, NSRunLoop};

use crate::geo::Coordinates;

const SLICE: Duration = Duration::from_millis(200);

/// Asks for when-in-use access, then waits up to `timeout` for a fix.
/// CoreLocation answers through the calling thread's run loop, so call it off
/// the UI thread.
pub(super) fn current_point(timeout: Duration) -> Option<Coordinates> {
    // SAFETY: the manager is created, used and dropped on this thread.
    unsafe {
        if !CLLocationManager::locationServicesEnabled_class() {
            log::info!("Location Services are off");
            return None;
        }
        let manager = CLLocationManager::new();
        manager.setDesiredAccuracy(kCLLocationAccuracyKilometer);
        manager.requestWhenInUseAuthorization();
        manager.startUpdatingLocation();
        let point = wait_for_fix(&manager, Instant::now() + timeout);
        manager.stopUpdatingLocation();
        point
    }
}

unsafe fn wait_for_fix(manager: &CLLocationManager, deadline: Instant) -> Option<Coordinates> {
    let run_loop = NSRunLoop::currentRunLoop();
    while Instant::now() < deadline {
        if refused(authorization()) {
            log::info!("Location access is denied");
            return None;
        }
        if let Some(location) = manager.location() {
            let coordinate = location.coordinate();
            return Some(Coordinates::new(coordinate.latitude, coordinate.longitude));
        }
        let slice_end = Instant::now() + SLICE;
        run_loop.runUntilDate(&NSDate::dateWithTimeIntervalSinceNow(SLICE.as_secs_f64()));
        // The run loop returns at once while it has no input source yet.
        thread::sleep(slice_end.saturating_duration_since(Instant::now()));
    }
    log::info!("No location fix before the timeout");
    None
}

// The instance method needs macOS 11, and the app still runs on older versions.
#[allow(deprecated)]
unsafe fn authorization() -> CLAuthorizationStatus {
    CLLocationManager::authorizationStatus_class()
}

fn refused(status: CLAuthorizationStatus) -> bool {
    status == CLAuthorizationStatus::Denied || status == CLAuthorizationStatus::Restricted
}
