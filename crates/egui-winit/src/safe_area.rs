#[cfg(target_os = "ios")]
pub use ios::get_safe_area_insets;

#[cfg(target_os = "ios")]
mod ios {
    use std::ptr::NonNull;
    use std::sync::{Mutex, Once, PoisonError};

    use block2::RcBlock;
    use egui::{SafeAreaInsets, epaint::MarginF32};
    use objc2::{ClassType, rc::Retained};
    use objc2_foundation::{
        MainThreadMarker, NSNotification, NSNotificationCenter, NSObjectProtocol, NSValue,
    };
    use objc2_ui_kit::{
        NSValueUIGeometryExtensions as _, UIApplication, UIKeyboardFrameEndUserInfoKey,
        UIKeyboardWillChangeFrameNotification, UISceneActivationState, UIWindowScene,
    };

    /// Where the on-screen keyboard's top edge is (screen points), as it last moved; `None` before
    /// it first shows. It is off the bottom of the screen while the keyboard is away.
    static KEYBOARD_TOP: Mutex<Option<f64>> = Mutex::new(None);

    /// Follow the on-screen keyboard: UIKit says where it is going whenever it moves.
    fn watch_keyboard() {
        static WATCH: Once = Once::new();
        WATCH.call_once(|| {
            let block = RcBlock::new(|notification: NonNull<NSNotification>| {
                // SAFETY: UIKit passes a live notification for the duration of the call.
                #[expect(unsafe_code)]
                let notification = unsafe { notification.as_ref() };
                // SAFETY: the keyboard notifications' user info holds an `NSValue` with a
                // `CGRect` under `UIKeyboardFrameEndUserInfoKey` (`downcast` checks the class).
                #[expect(unsafe_code)]
                let top = unsafe {
                    notification
                        .userInfo()
                        .and_then(|info| info.objectForKey(UIKeyboardFrameEndUserInfoKey))
                        .and_then(|value| value.downcast::<NSValue>().ok())
                        .map(|value| value.CGRectValue().origin.y)
                };
                if let Some(top) = top {
                    *KEYBOARD_TOP.lock().unwrap_or_else(PoisonError::into_inner) = Some(top);
                }
            });
            // SAFETY: the block lives as long as the observer, which lives as long as the app
            // (the token is kept on purpose: the keyboard is followed until the app ends).
            #[expect(unsafe_code)]
            let observer = unsafe {
                NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
                    Some(UIKeyboardWillChangeFrameNotification),
                    None,
                    None,
                    &block,
                )
            };
            std::mem::forget(observer);
        });
    }

    /// Gets the ios safe area insets.
    ///
    /// A safe area defines the area within a view that isn’t covered by a navigation bar, tab bar,
    /// toolbar, or other views a window might provide. Safe areas are essential for avoiding a
    /// device’s interactive and display features, like Dynamic Island on iPhone or the camera
    /// housing on some Mac models.
    ///
    /// Once winit v0.31 has been released this can be removed in favor of
    /// `winit::Window::safe_area`.
    pub fn get_safe_area_insets() -> SafeAreaInsets {
        let Some(main_thread_marker) = MainThreadMarker::new() else {
            log::error!("Getting safe area insets needs to be performed on the main thread");
            return SafeAreaInsets::default();
        };

        watch_keyboard();
        let app = UIApplication::sharedApplication(main_thread_marker);

        #[expect(unsafe_code)]
        unsafe {
            // Look for the first window scene that's in the foreground
            for scene in app.connectedScenes() {
                if scene.isKindOfClass(UIWindowScene::class())
                    && matches!(
                        scene.activationState(),
                        UISceneActivationState::ForegroundActive
                            | UISceneActivationState::ForegroundInactive
                    )
                {
                    // SAFETY: class kind was checked above with `isKindOfClass`
                    let window_scene = Retained::cast_unchecked::<UIWindowScene>(scene.clone());
                    if let Some(window) = window_scene.keyWindow() {
                        let insets = window.safeAreaInsets();
                        // The on-screen keyboard counts as covered too, so what the app lays
                        // out stays above it.
                        let bounds = window.bounds();
                        let under_keyboard = KEYBOARD_TOP
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .map_or(0.0, |top| bounds.origin.y + bounds.size.height - top)
                            .clamp(0.0, bounds.size.height);
                        return SafeAreaInsets(MarginF32 {
                            top: insets.top as f32,
                            left: insets.left as f32,
                            right: insets.right as f32,
                            bottom: insets.bottom.max(under_keyboard) as f32,
                        });
                    }
                }
            }
        }

        SafeAreaInsets::default()
    }
}
