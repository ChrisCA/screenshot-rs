//! Capture a bitmap image of a display. The resulting screenshot is stored in
//! the `Screenshot` type, which varies per platform.
//!
//! # Platform-specific details
//!
//! Despite OS X's CoreGraphics documentation, the bitmap returned has its
//! origin at the top left corner. It uses ARGB pixels.
//!
//! The Windows GDI bitmap has its coordinate origin at the bottom left. We
//! attempt to undo this by reordering the rows. Windows also uses ARGB pixels.

mod bindings {
    windows::include_bindings!();
}

use bindings::{
    Windows::Data::Xml::Dom::*,
    Windows::Win32::Foundation::CloseHandle,
    Windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForSingleObject},
    Windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_OK},
};

use std::mem::size_of;

//use std::intrinsics::{size_of, offset};
pub use ffi::get_screenshot;

#[derive(Clone, Copy)]
pub struct Pixel {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// An image buffer containing the screenshot.
/// Pixels are stored as [ARGB](https://en.wikipedia.org/wiki/ARGB).
pub struct Screenshot {
    data: Vec<u8>,
    height: usize,
    width: usize,
    row_len: usize, // Might be superfluous
    pixel_width: usize,
}

impl Screenshot {
    /// Height of image in pixels.
    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    /// Width of image in pixels.
    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Number of bytes in one row of bitmap.
    #[inline]
    pub fn row_len(&self) -> usize {
        self.row_len
    }

    /// Width of pixel in bytes.
    #[inline]
    pub fn pixel_width(&self) -> usize {
        self.pixel_width
    }

    /// Raw bitmap.
    #[inline]
    pub unsafe fn raw_data(&self) -> *const u8 {
        &self.data[0] as *const u8
    }

    /// Raw bitmap.
    #[inline]
    pub unsafe fn raw_data_mut(&mut self) -> *mut u8 {
        &mut self.data[0] as *mut u8
    }

    /// Number of bytes in bitmap
    #[inline]
    pub fn raw_len(&self) -> usize {
        self.data.len() * unsafe { size_of::<u8>() }
    }

    // /// Gets pixel at (row, col)
    // pub fn get_pixel(&self, row: usize, col: usize) -> Pixel {
    //     let idx = (row * self.row_len() + col * self.pixel_width()) as isize;
    //     unsafe {
    //         let data = &self.data[0] as *const u8;
    //         if idx as usize > self.raw_len() {
    //             panic!("Bounds overflow");
    //         }

    //         Pixel {
    //             a: *offset(data, idx + 3),
    //             r: *offset(data, idx + 2),
    //             g: *offset(data, idx + 1),
    //             b: *offset(data, idx),
    //         }
    //     }
    // }

	// TODO: not sure if this was correctly tranferred
    /// Gets pixel at (row, col)
    pub fn get_pixel(&self, row: usize, col: usize) -> Pixel {
        let idx = row * self.row_len() + col * self.pixel_width();
        let data = &self.data;
        if idx as usize > self.raw_len() {
            panic!("Bounds overflow");
        }

        Pixel {
            a: data[idx + 3],
            r: data[idx + 2],
            g: data[idx + 1],
            b: data[idx],
        }
    }
}

impl AsRef<[u8]> for Screenshot {
    #[inline]
    fn as_ref<'a>(&'a self) -> &'a [u8] {
        self.data.as_slice()
    }
}

pub type ScreenResult = Result<Screenshot, &'static str>;

#[cfg(target_os = "windows")]
mod ffi {
    #![allow(non_snake_case, dead_code)]

    use std::mem::size_of;

    use libc::{c_int, c_long, c_uint, c_void};

    use ScreenResult;
    use Screenshot;

    type PVOID = *mut c_void;
    type LPVOID = *mut c_void;
    type WORD = u16; // c_uint;
    type DWORD = u32; // c_ulong;
    type BOOL = c_int;
    type BYTE = u8;
    type UINT = c_uint;
    type LONG = c_long;
    type LPARAM = c_long;

    #[repr(C)]
    struct RECT {
        left: LONG,
        top: LONG,
        right: LONG,  // immediately outside rect
        bottom: LONG, // immediately outside rect
    }
    type LPCRECT = *const RECT;
    type LPRECT = *mut RECT;

    type HANDLE = PVOID;
    type HMONITOR = HANDLE;
    type HWND = HANDLE;
    type HDC = HANDLE;
    #[repr(C)]
    struct MONITORINFO {
        cbSize: DWORD,
        rcMonitor: RECT,
        rcWork: RECT,
        dwFlags: DWORD,
    }
    type LPMONITORINFO = *mut MONITORINFO;
    type MONITORENUMPROC = fn(HMONITOR, HDC, LPRECT, LPARAM) -> BOOL;

    type HBITMAP = HANDLE;
    type HGDIOBJ = HANDLE;
    type LPBITMAPINFO = PVOID; // Hack

    const NULL: *mut c_void = 0usize as *mut c_void;
    const HGDI_ERROR: *mut c_void = -1isize as *mut c_void;
    const SM_CXSCREEN: c_int = 0;
    const SM_CYSCREEN: c_int = 1;

    /// TODO verify value
    const SRCCOPY: u32 = 0x00CC0020;
    const CAPTUREBLT: u32 = 0x40000000;
    const DIB_RGB_COLORS: UINT = 0;
    const BI_RGB: DWORD = 0;

    #[repr(C)]
    struct BITMAPINFOHEADER {
        biSize: DWORD,
        biWidth: LONG,
        biHeight: LONG,
        biPlanes: WORD,
        biBitCount: WORD,
        biCompression: DWORD,
        biSizeImage: DWORD,
        biXPelsPerMeter: LONG,
        biYPelsPerMeter: LONG,
        biClrUsed: DWORD,
        biClrImportant: DWORD,
    }

    #[repr(C)]
    struct RGBQUAD {
        rgbBlue: BYTE,
        rgbGreen: BYTE,
        rgbRed: BYTE,
        rgbReserved: BYTE,
    }

    /// WARNING variable sized struct
    #[repr(C)]
    struct BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER,
        bmiColors: [RGBQUAD; 1],
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetSystemMetrics(m: c_int) -> c_int;
        fn EnumDisplayMonitors(
            hdc: HDC,
            lprcClip: LPCRECT,
            lpfnEnum: MONITORENUMPROC,
            dwData: LPARAM,
        ) -> BOOL;
        fn GetMonitorInfo(hMonitor: HMONITOR, lpmi: LPMONITORINFO) -> BOOL;
        fn GetDesktopWindow() -> HWND;
        fn GetDC(hWnd: HWND) -> HDC;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateCompatibleDC(hdc: HDC) -> HDC;
        fn CreateCompatibleBitmap(hdc: HDC, nWidth: c_int, nHeight: c_int) -> HBITMAP;
        fn SelectObject(hdc: HDC, hgdiobj: HGDIOBJ) -> HGDIOBJ;
        fn BitBlt(
            hdcDest: HDC,
            nXDest: c_int,
            nYDest: c_int,
            nWidth: c_int,
            nHeight: c_int,
            hdcSrc: HDC,
            nXSrc: c_int,
            nYSrc: c_int,
            dwRop: DWORD,
        ) -> BOOL;
        fn GetDIBits(
            hdc: HDC,
            hbmp: HBITMAP,
            uStartScan: UINT,
            cScanLines: UINT,
            lpvBits: LPVOID,
            lpbi: LPBITMAPINFO,
            uUsage: UINT,
        ) -> c_int;

        fn DeleteObject(hObject: HGDIOBJ) -> BOOL;
        fn ReleaseDC(hWnd: HWND, hDC: HDC) -> c_int;
        fn DeleteDC(hdc: HDC) -> BOOL;
    }

    /// Reorder rows in bitmap, last to first.
    /// TODO rewrite functionally
    fn flip_rows(data: Vec<u8>, height: usize, row_len: usize) -> Vec<u8> {
        let mut new_data = Vec::with_capacity(data.len());
        unsafe { new_data.set_len(data.len()) };
        for row_i in 0..height {
            for byte_i in 0..row_len {
                let old_idx = (height - row_i - 1) * row_len + byte_i;
                let new_idx = row_i * row_len + byte_i;
                new_data[new_idx] = data[old_idx];
            }
        }
        new_data
    }

    /// TODO Support multiple screens
    /// This may never happen, given the horrific quality of Win32 APIs
    pub fn get_screenshot(_screen: usize) -> ScreenResult {
        unsafe {
            // Enumerate monitors, getting a handle and DC for requested monitor.
            // loljk, because doing that on Windows is worse than death
            let h_wnd_screen = GetDesktopWindow();
            let h_dc_screen = GetDC(h_wnd_screen);
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);

            // Create a Windows Bitmap, and copy the bits into it
            let h_dc = CreateCompatibleDC(h_dc_screen);
            if h_dc == NULL {
                return Err("Can't get a Windows display.");
            }

            let h_bmp = CreateCompatibleBitmap(h_dc_screen, width, height);
            if h_bmp == NULL {
                return Err("Can't create a Windows buffer");
            }

            let res = SelectObject(h_dc, h_bmp);
            if res == NULL || res == HGDI_ERROR {
                return Err("Can't select Windows buffer.");
            }

            let res = BitBlt(
                h_dc,
                0,
                0,
                width,
                height,
                h_dc_screen,
                0,
                0,
                SRCCOPY | CAPTUREBLT,
            );
            if res == 0 {
                return Err("Failed to copy screen to Windows buffer");
            }

            // Get image info
            let pixel_width: usize = 4; // FIXME
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as DWORD,
                    biWidth: width as LONG,
                    biHeight: height as LONG,
                    biPlanes: 1,
                    biBitCount: 8 * pixel_width as WORD,
                    biCompression: BI_RGB,
                    biSizeImage: (width * height * pixel_width as c_int) as DWORD,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD {
                    rgbBlue: 0,
                    rgbGreen: 0,
                    rgbRed: 0,
                    rgbReserved: 0,
                }],
            };

            // Create a Vec for image
            let size: usize = (width * height) as usize * pixel_width;
            let mut data: Vec<u8> = Vec::with_capacity(size);
            data.set_len(size);

            // copy bits into Vec
            GetDIBits(
                h_dc,
                h_bmp,
                0,
                height as DWORD,
                &mut data[0] as *mut u8 as *mut c_void,
                &mut bmi as *mut BITMAPINFO as *mut c_void,
                DIB_RGB_COLORS,
            );

            // Release native image buffers
            ReleaseDC(h_wnd_screen, h_dc_screen); // don't need screen anymore
            DeleteDC(h_dc);
            DeleteObject(h_bmp);

            let data = flip_rows(data, height as usize, width as usize * pixel_width);

            Ok(Screenshot {
                data,
                height: height as usize,
                width: width as usize,
                row_len: width as usize * pixel_width,
                pixel_width,
            })
        }
    }
}

#[test]
fn test_get_screenshot() {
    let s: Screenshot = get_screenshot(0).unwrap();
    println!(
        "width: {}\n height: {}\npixel width: {}\n bytes: {}",
        s.width(),
        s.height(),
        s.pixel_width(),
        s.raw_len()
    );
}
