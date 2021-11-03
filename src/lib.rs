//! Capture a bitmap image of a display. The resulting screenshot is stored in
//! the `Screenshot` type, which varies per platform.
//!
//! The Windows GDI bitmap has its coordinate origin at the bottom left. We
//! attempt to undo this by reordering the rows. Windows also uses ARGB pixels.

use windows::{Win32::Graphics::Gdi::*, Win32::UI::WindowsAndMessaging::*};

use core::ffi::c_void;
use std::{error::Error, mem::size_of};

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
    pub data: Vec<u8>,
    /// Height of image in pixels
    pub height: usize,
    /// Width of image in pixels.
    pub width: usize,
    /// Number of bytes in one row of bitmap.
    pub row_len: usize, // Might be superfluous
    /// Width of pixel in bytes.
    pub pixel_width: usize,
}

impl Screenshot {
    /// Number of bytes in bitmap
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets pixel at (row, col)
    pub fn get_pixel(&self, row: usize, col: usize) -> Pixel {
        let idx = row * self.row_len + col * self.pixel_width;
        let data = &self.data;
        if idx as usize > self.len() {
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
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

// TODO: this needs probably a rewrite, without this the image is mirror over the center point
/// Reorder rows in bitmap, last to first
fn flip_rows(data: Vec<u8>, height: usize, row_len: usize) -> Vec<u8> {
    let mut new_data = Vec::with_capacity(data.len());
    new_data.resize_with(data.len(), Default::default);

    for row_i in 0..height {
        for byte_i in 0..row_len {
            let old_idx = (height - row_i - 1) * row_len + byte_i;
            let new_idx = row_i * row_len + byte_i;
            new_data[new_idx] = data[old_idx];
        }
    }

    new_data
}

// TODO: Support multiple screens
// gets a screenshot from a default screen
pub fn get_screenshot() -> Result<Screenshot, Box<dyn Error>> {
    unsafe {
        // Enumerate monitors, getting a handle and DC for requested monitor.
        // loljk, because doing that on Windows is worse than death
        let h_wnd_screen = GetDesktopWindow();
        let h_dc_screen = GetDC(h_wnd_screen);
        let width = GetSystemMetrics(SYSTEM_METRICS_INDEX(SM_CXSCREEN.0));
        let height = GetSystemMetrics(SYSTEM_METRICS_INDEX(SM_CYSCREEN.0));

        // Create a Windows Bitmap, and copy the bits into it
        let h_dc = CreateCompatibleDC(h_dc_screen);
        let h_bmp = CreateCompatibleBitmap(h_dc_screen, width, height);
        let _ = SelectObject(h_dc, h_bmp);

        let res = BitBlt(
            h_dc,
            0,
            0,
            width,
            height,
            h_dc_screen,
            0,
            0,
            ROP_CODE(SRCCOPY.0 | CAPTUREBLT.0),
        );

        if !res.as_bool() {
            return Err("Failed to copy screen to Windows buffer".into());
        }

        // Get image info
        let pixel_width: usize = 4; // FIXME
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: height,
                biPlanes: 1,
                biBitCount: 8 * pixel_width as u16,
                biCompression: BI_RGB as u32,
                biSizeImage: (width * height * (pixel_width as i32)) as u32,
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
        data.resize_with(size, Default::default);

        // copy bits into Vec
        GetDIBits(
            h_dc,
            h_bmp,
            0,
            height as u32,
            &mut data[0] as *mut u8 as *mut c_void,
            &mut bmi as *mut BITMAPINFO,
            DIB_USAGE(DIB_RGB_COLORS.0),
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

#[test]
fn test_get_screenshot() {
    let s: Screenshot = get_screenshot().unwrap();
    println!(
        "width: {}\n height: {}\npixel width: {}\n bytes: {}",
        s.width,
        s.height,
        s.pixel_width,
        s.len()
    );
}
