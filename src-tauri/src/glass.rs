//! macOS: the system material behind Peron's transparent WebView (see styles.macos.css).
//! Liquid Glass (`NSGlassEffectView`, macOS 26+) when the class exists, the classic sidebar
//! vibrancy (`NSVisualEffectView`) before that. The material view is inserted below every other
//! subview of the window's content view, i.e. behind the WebView, and resizes with the window.

use objc2::rc::Retained;
use objc2::runtime::AnyClass;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSGlassEffectView, NSGlassEffectViewStyle, NSView,
    NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView,
    NSWindow, NSWindowOrderingMode,
};

/// Must run on the main thread (AppKit). Returns which material was applied.
pub fn apply(window: &tauri::WebviewWindow) -> Result<&'static str, String> {
    let mtm = MainThreadMarker::new().ok_or("not on the main thread")?;
    let ptr = window.ns_window().map_err(|e| e.to_string())?;
    // SAFETY: Tauri hands out the live NSWindow of this webview window; we only borrow it here,
    // on the main thread, while `window` keeps it alive.
    let ns_window: &NSWindow = unsafe { &*ptr.cast::<NSWindow>() };
    let content = ns_window
        .contentView()
        .ok_or("window has no content view")?;
    let frame = content.bounds();

    let (view, kind): (Retained<NSView>, _) = if AnyClass::get(c"NSGlassEffectView").is_some() {
        let glass = NSGlassEffectView::initWithFrame(mtm.alloc(), frame);
        glass.setStyle(NSGlassEffectViewStyle::Regular);
        (Retained::into_super(glass), "liquid glass")
    } else {
        let vibrancy = NSVisualEffectView::initWithFrame(mtm.alloc(), frame);
        vibrancy.setMaterial(NSVisualEffectMaterial::Sidebar);
        vibrancy.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        vibrancy.setState(NSVisualEffectState::FollowsWindowActiveState);
        (Retained::into_super(vibrancy), "vibrancy")
    };
    view.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    content.addSubview_positioned_relativeTo(&view, NSWindowOrderingMode::Below, None);
    Ok(kind)
}
