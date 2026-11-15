//! iOS: putting winit's window in the app's window scene.
//!
//! iPadOS 27 doesn't start an app built with its SDK unless the app adopts the scene life cycle
//! (an `UIApplicationSceneManifest` in its `Info.plist`). A window of such an app is shown only
//! in a scene, and winit makes its `UIWindow` with `initWithFrame:`, in none: the app runs, but
//! the screen stays black. [`adopt_window_scene`] moves the window into the app's first window
//! scene once one is connected.

/// Put `window` in the app's window scene, if it is in none and a scene is connected (from the
/// app becoming active on). Whether the window is in a scene afterwards.
#[cfg(target_os = "ios")]
pub fn adopt_window_scene(window: &winit::window::Window) -> bool {
    use objc2::{ClassType as _, rc::Retained};
    use objc2_foundation::{MainThreadMarker, NSObjectProtocol as _};
    use objc2_ui_kit::{UIApplication, UIView, UIWindowScene};
    use raw_window_handle::{HasWindowHandle as _, RawWindowHandle};

    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    let Ok(RawWindowHandle::UiKit(handle)) = window.window_handle().map(|h| h.as_raw()) else {
        return false;
    };
    // SAFETY: winit's UIKit handle points to its window's root view, which lives as long as the
    // window does; this runs on the main thread.
    #[expect(unsafe_code)]
    let view: &UIView = unsafe { handle.ui_view.cast::<UIView>().as_ref() };
    let Some(ui_window) = view.window() else {
        return false;
    };
    if ui_window.windowScene().is_some() {
        return true;
    }
    let app = UIApplication::sharedApplication(mtm);
    for scene in app.connectedScenes() {
        if scene.isKindOfClass(UIWindowScene::class()) {
            // SAFETY: the class was checked with `isKindOfClass` just above
            #[expect(unsafe_code)]
            let scene = unsafe { Retained::cast_unchecked::<UIWindowScene>(scene) };
            ui_window.setWindowScene(Some(&scene));
            ui_window.makeKeyAndVisible();
            log::debug!("the window is in the app's window scene");
            return true;
        }
    }
    false
}

/// Other platforms have no window scenes.
#[cfg(not(target_os = "ios"))]
pub fn adopt_window_scene(_window: &winit::window::Window) -> bool {
    true
}
