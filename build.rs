fn main() {
    windows::build! {
        Windows::Win32::UI::WindowsAndMessaging::*,
        Windows::Win32::Graphics::Gdi::*,
    };
}
