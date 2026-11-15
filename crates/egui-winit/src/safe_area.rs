#[cfg(target_os = "ios")]
pub use ios::get_safe_area_insets;

#[cfg(target_os = "ios")]
mod ios {
    use egui::{SafeAreaInsets, epaint::MarginF32};
    use objc2::{ClassType, rc::Retained};
    use objc2_foundation::{MainThreadMarker, NSObjectProtocol};
    use objc2_ui_kit::{UIApplication, UISceneActivationState, UIWindowScene};

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
                        // out stays above it (the keyboard layout guide sits on the safe area's
                        // bottom edge while the keyboard is away).
                        let bounds = window.bounds();
                        let keyboard = window.keyboardLayoutGuide().layoutFrame();
                        let under_keyboard =
                            bounds.origin.y + bounds.size.height - keyboard.origin.y;
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
