use std::os::raw::{c_uint, c_ulong};
use std::ptr;

use crate::capturable::Capturable;
use crate::input::device::{InputDevice, InputDeviceType};
use crate::protocol::{
    Button, KeyboardEvent, KeyboardEventType, KeyboardLocation, PointerEvent, PointerEventType,
    PointerType, WheelEvent,
};

use tracing::{debug, warn};

// X11 bindings
use x11::xlib::{
    Display, False, True, XCloseDisplay, XDefaultScreen, XDisplayHeight, XDisplayWidth, XFlush,
    XKeysymToKeycode, XOpenDisplay, XSync,
};
use x11::xtest::{XTestFakeButtonEvent, XTestFakeKeyEvent, XTestFakeMotionEvent, XTestQueryExtension};

pub struct XTestDevice {
    display: *mut Display,
    capturable: Box<dyn Capturable>,
}

impl XTestDevice {
    pub fn new(capturable: Box<dyn Capturable>) -> Result<Self, String> {
        unsafe {
            // Open connection to X server
            let display = XOpenDisplay(ptr::null());
            if display.is_null() {
                return Err("Failed to open X display. Make sure DISPLAY environment variable is set and X server is running.".to_string());
            }

            // Check if XTEST extension is available
            let mut event_base = 0;
            let mut error_base = 0;
            let mut major_version = 0;
            let mut minor_version = 0;

            let xtest_available = XTestQueryExtension(
                display,
                &mut event_base,
                &mut error_base,
                &mut major_version,
                &mut minor_version,
            );

            if xtest_available == 0 {
                XCloseDisplay(display);
                return Err("XTEST extension is not available on this X server.".to_string());
            }

            debug!(
                "XTEST extension available: version {}.{}",
                major_version, minor_version
            );

            Ok(Self {
                display,
                capturable,
            })
        }
    }

    fn send_key(&mut self, keysym: c_ulong, is_press: bool) {
        if keysym == 0 {
            return;
        }

        unsafe {
            // Convert KeySym to KeyCode
            let keycode = XKeysymToKeycode(self.display, keysym);
            if keycode == 0 {
                warn!("Cannot convert keysym 0x{:x} to keycode", keysym);
                return;
            }

            // Send the key event using XTest
            XTestFakeKeyEvent(
                self.display,
                keycode as u32,
                if is_press { True } else { False },
                0, // CurrentTime
            );
            XFlush(self.display);
        }
    }

    fn screen_coordinates(&self, x: f64, y: f64) -> Option<(i32, i32)> {
        let (x_rel, y_rel, width_rel, height_rel) = match self.capturable.geometry().ok()? {
            crate::capturable::Geometry::Relative(x, y, width, height) => (x, y, width, height),
        };

        unsafe {
            let screen = XDefaultScreen(self.display);
            let width = XDisplayWidth(self.display, screen);
            let height = XDisplayHeight(self.display, screen);
            if width <= 0 || height <= 0 {
                return None;
            }

            let x = ((x * width_rel + x_rel) * width as f64).round() as i32;
            let y = ((y * height_rel + y_rel) * height as f64).round() as i32;

            Some((x.clamp(0, width - 1), y.clamp(0, height - 1)))
        }
    }

    fn move_pointer(&mut self, x: f64, y: f64) {
        let Some((x, y)) = self.screen_coordinates(x, y) else {
            warn!("Failed to determine target coordinates for XTest pointer event");
            return;
        };

        unsafe {
            let screen = XDefaultScreen(self.display);
            XTestFakeMotionEvent(self.display, screen, x, y, 0);
            XFlush(self.display);
        }
    }

    fn button_number(button: Button) -> Option<c_uint> {
        match button {
            Button::PRIMARY => Some(1),
            Button::AUXILARY => Some(2),
            Button::SECONDARY => Some(3),
            Button::FOURTH => Some(8),
            Button::FIFTH => Some(9),
            _ => None,
        }
    }

    fn send_button(&mut self, button: c_uint, is_press: bool) {
        unsafe {
            XTestFakeButtonEvent(
                self.display,
                button,
                if is_press { True } else { False },
                0,
            );
            XFlush(self.display);
        }
    }

    fn release_all_mouse_buttons(&mut self) {
        for button in [1, 2, 3, 8, 9] {
            self.send_button(button, false);
        }
    }

    fn click_button(&mut self, button: c_uint, repeat: u32) {
        for _ in 0..repeat {
            self.send_button(button, true);
            self.send_button(button, false);
        }
    }
}

impl Drop for XTestDevice {
    fn drop(&mut self) {
        unsafe {
            if !self.display.is_null() {
                XCloseDisplay(self.display);
            }
        }
    }
}

impl InputDevice for XTestDevice {
    fn send_keyboard_event(&mut self, event: &KeyboardEvent) {
        use crate::input::x11_keys::*;

        if let Err(err) = self.capturable.before_input() {
            warn!("Failed to activate window, sending no input ({})", err);
            return;
        }

        fn map_key(code: &str, location: &KeyboardLocation, shift: bool) -> c_ulong {
            match (code, location, shift) {
                ("Escape", _, _) => XK_ESCAPE,

                // Numbers
                ("Digit0", KeyboardLocation::NUMPAD, _) => XK_KP_0,
                ("Digit1", KeyboardLocation::NUMPAD, _) => XK_KP_1,
                ("Digit2", KeyboardLocation::NUMPAD, _) => XK_KP_2,
                ("Digit3", KeyboardLocation::NUMPAD, _) => XK_KP_3,
                ("Digit4", KeyboardLocation::NUMPAD, _) => XK_KP_4,
                ("Digit5", KeyboardLocation::NUMPAD, _) => XK_KP_5,
                ("Digit6", KeyboardLocation::NUMPAD, _) => XK_KP_6,
                ("Digit7", KeyboardLocation::NUMPAD, _) => XK_KP_7,
                ("Digit8", KeyboardLocation::NUMPAD, _) => XK_KP_8,
                ("Digit9", KeyboardLocation::NUMPAD, _) => XK_KP_9,
                ("Digit0", _, _) => XK_0,
                ("Digit1", _, _) => XK_1,
                ("Digit2", _, _) => XK_2,
                ("Digit3", _, _) => XK_3,
                ("Digit4", _, _) => XK_4,
                ("Digit5", _, _) => XK_5,
                ("Digit6", _, _) => XK_6,
                ("Digit7", _, _) => XK_7,
                ("Digit8", _, _) => XK_8,
                ("Digit9", _, _) => XK_9,

                // Letters - lowercase by default, uppercase when shift is pressed is handled by X11
                ("KeyA", _, _) => XK_a,
                ("KeyB", _, _) => XK_b,
                ("KeyC", _, _) => XK_c,
                ("KeyD", _, _) => XK_d,
                ("KeyE", _, _) => XK_e,
                ("KeyF", _, _) => XK_f,
                ("KeyG", _, _) => XK_g,
                ("KeyH", _, _) => XK_h,
                ("KeyI", _, _) => XK_i,
                ("KeyJ", _, _) => XK_j,
                ("KeyK", _, _) => XK_k,
                ("KeyL", _, _) => XK_l,
                ("KeyM", _, _) => XK_m,
                ("KeyN", _, _) => XK_n,
                ("KeyO", _, _) => XK_o,
                ("KeyP", _, _) => XK_p,
                ("KeyQ", _, _) => XK_q,
                ("KeyR", _, _) => XK_r,
                ("KeyS", _, _) => XK_s,
                ("KeyT", _, _) => XK_t,
                ("KeyU", _, _) => XK_u,
                ("KeyV", _, _) => XK_v,
                ("KeyW", _, _) => XK_w,
                ("KeyX", _, _) => XK_x,
                ("KeyY", _, _) => XK_y,
                ("KeyZ", _, _) => XK_z,

                // Function keys
                ("F1", _, _) => XK_F1,
                ("F2", _, _) => XK_F2,
                ("F3", _, _) => XK_F3,
                ("F4", _, _) => XK_F4,
                ("F5", _, _) => XK_F5,
                ("F6", _, _) => XK_F6,
                ("F7", _, _) => XK_F7,
                ("F8", _, _) => XK_F8,
                ("F9", _, _) => XK_F9,
                ("F10", _, _) => XK_F10,
                ("F11", _, _) => XK_F11,
                ("F12", _, _) => XK_F12,

                // Special keys
                ("Backspace", _, _) => XK_BACKSPACE,
                ("Tab", _, _) => XK_TAB,
                ("Enter", KeyboardLocation::NUMPAD, _) => XK_KP_ENTER,
                ("Enter", _, _) => XK_RETURN,
                ("Space", _, _) => XK_SPACE,
                ("CapsLock", _, _) => XK_CAPS_LOCK,
                ("NumLock", _, _) => XK_NUM_LOCK,
                ("ScrollLock", _, _) => XK_SCROLL_LOCK,
                ("Pause", _, _) => XK_PAUSE,
                ("Insert", _, _) => XK_INSERT,
                ("Delete", _, _) => XK_DELETE,
                ("Home", _, _) => XK_HOME,
                ("End", _, _) => XK_END,
                ("PageUp", _, _) => XK_PAGE_UP,
                ("PageDown", _, _) => XK_PAGE_DOWN,

                // Arrow keys
                ("ArrowLeft", _, _) => XK_LEFT,
                ("ArrowUp", _, _) => XK_UP,
                ("ArrowRight", _, _) => XK_RIGHT,
                ("ArrowDown", _, _) => XK_DOWN,

                // Punctuation
                ("Minus", KeyboardLocation::NUMPAD, _) => XK_KP_SUBTRACT,
                ("Equal", KeyboardLocation::NUMPAD, _) => XK_KP_EQUAL,
                ("Minus", _, _) => XK_MINUS,
                ("Equal", _, _) => XK_EQUAL,
                ("BracketLeft", _, _) => XK_BRACKETLEFT,
                ("BracketRight", _, _) => XK_BRACKETRIGHT,
                ("Backslash", _, _) => XK_BACKSLASH,
                ("Semicolon", _, _) => XK_SEMICOLON,
                ("Quote", _, _) => XK_APOSTROPHE,
                ("Backquote", _, _) => XK_GRAVE,
                ("Comma", _, _) => XK_COMMA,
                ("Period", _, _) => XK_PERIOD,
                ("Slash", _, _) => XK_SLASH,

                // Numpad
                ("NumpadMultiply", _, _) => XK_KP_MULTIPLY,
                ("NumpadAdd", _, _) => XK_KP_ADD,
                ("NumpadSubtract", _, _) => XK_KP_SUBTRACT,
                ("NumpadDecimal", _, _) => XK_KP_DECIMAL,
                ("NumpadDivide", _, _) => XK_KP_DIVIDE,

                // Unknown key
                _ => {
                    warn!("Unknown key code: {}", code);
                    0
                }
            }
        }

        let key_code = map_key(&event.code, &event.location, event.shift);
        let is_press = match event.event_type {
            KeyboardEventType::UP => false,
            KeyboardEventType::DOWN => true,
            KeyboardEventType::REPEAT => true, // Treat repeat as press
        };

        // Send modifier keys first (for DOWN events)
        if is_press {
            if event.shift {
                self.send_key(XK_SHIFT_L, true);
            }
            if event.ctrl {
                self.send_key(XK_CONTROL_L, true);
            }
            if event.alt {
                self.send_key(XK_ALT_L, true);
            }
            if event.meta {
                self.send_key(XK_SUPER_L, true);
            }
        }

        // Send the main key
        self.send_key(key_code, is_press);

        // Release modifier keys (for UP events)
        if !is_press {
            if event.shift {
                self.send_key(XK_SHIFT_L, false);
            }
            if event.ctrl {
                self.send_key(XK_CONTROL_L, false);
            }
            if event.alt {
                self.send_key(XK_ALT_L, false);
            }
            if event.meta {
                self.send_key(XK_SUPER_L, false);
            }
        }

        // Flush to ensure events are sent
        unsafe {
            XSync(self.display, False);
        }
    }

    fn send_pointer_event(&mut self, event: &PointerEvent) {
        if !event.is_primary && !matches!(event.pointer_type, PointerType::Mouse) {
            return;
        }

        if let Err(err) = self.capturable.before_input() {
            warn!("Failed to activate window, sending no input ({})", err);
            return;
        }

        match event.event_type {
            PointerEventType::MOVE | PointerEventType::OVER | PointerEventType::ENTER => {
                self.move_pointer(event.x, event.y);
            }
            PointerEventType::DOWN => {
                self.move_pointer(event.x, event.y);
                if let Some(button) = Self::button_number(event.button) {
                    self.send_button(button, true);
                }
            }
            PointerEventType::UP => {
                self.move_pointer(event.x, event.y);
                if let Some(button) = Self::button_number(event.button) {
                    self.send_button(button, false);
                }
            }
            PointerEventType::CANCEL | PointerEventType::LEAVE | PointerEventType::OUT => {
                self.release_all_mouse_buttons();
            }
        }

        unsafe {
            XSync(self.display, False);
        }
    }

    fn send_wheel_event(&mut self, event: &WheelEvent) {
        if let Err(err) = self.capturable.before_input() {
            warn!("Failed to activate window, sending no input ({})", err);
            return;
        }

        if event.dy > 0 {
            self.click_button(4, 1);
        } else if event.dy < 0 {
            self.click_button(5, 1);
        }

        if event.dx > 0 {
            self.click_button(6, 1);
        } else if event.dx < 0 {
            self.click_button(7, 1);
        }

        unsafe {
            XSync(self.display, False);
        }
    }

    fn set_capturable(&mut self, capturable: Box<dyn Capturable>) {
        self.capturable = capturable;
    }

    fn device_type(&self) -> InputDeviceType {
        InputDeviceType::XTestDevice
    }
}
