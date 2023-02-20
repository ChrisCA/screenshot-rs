//! Capture a bitmap image of a display. The resulting screenshot is stored in
//! the `Screenshot` type, which varies per platform.
//!
//! The Windows GDI bitmap has its coordinate origin at the bottom left. We
//! attempt to undo this by reordering the rows. Windows also uses ARGB pixels.

use windows::{Win32::Graphics::Gdi::*, Win32::UI::WindowsAndMessaging::*};

use core::ffi::c_void;
use std::{error::Error, mem::size_of};

// 4 as 32 bit colour
const PIXEL_WIDTH: usize = 4;

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
    pub data_r_and_b_switched: Vec<u8>,
    /// Height of image in pixels
    pub height: usize,
    /// Width of image in pixels.
    pub width: usize,
    /// Number of bytes in one row of bitmap.
    pub row_len: usize, // Might be superfluous
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
        let idx = row * self.row_len + col * PIXEL_WIDTH;
        if idx > self.len() {
            panic!("Bounds overflow");
        }

        Pixel {
            a: self.data[idx + 3],
            r: self.data[idx + 2],
            g: self.data[idx + 1],
            b: self.data[idx],
        }
    }
}

// TODO: Support multiple screens
// gets a screenshot from a default screen
pub fn get_screenshot() -> Result<Screenshot, Box<dyn Error>> {
    unsafe {
        // Enumerate monitors, getting a handle and DC for requested monitor.
        // loljk, because doing that on Windows is worse than death
        let h_wnd_screen = GetDesktopWindow();
        let h_dc_screen = GetDC(h_wnd_screen);
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);

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
            ROP_CODE(SRCCOPY.0),
        );

        if !res.as_bool() {
            return Err("Failed to copy screen to Windows buffer".into());
        }

        // Get image info
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // having this reverted by -1 causes the image to be flipped to save a additional flipping step later
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0, // as compression is set to RGB, this may be set to zero (width * height * (pixel_width as i32)) as u32,
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
        let size: usize = (width * height) as usize * PIXEL_WIDTH;
        let mut data: Vec<u8> = vec![0; size];

        // copy bits into Vec
        GetDIBits(
            h_dc,
            h_bmp,
            0,
            height as u32,
            Some(&mut data[0] as *mut _ as *mut c_void),
            &mut bmi as *mut BITMAPINFO,
            DIB_RGB_COLORS,
        );

        // create a colour inverted version, switch r and b
        let mut data_color_invert = data.clone();
        let l = data_color_invert.len();
        for i in (0..l).into_iter().step_by(4) {
            data_color_invert.swap(i, i + 2);
        }

        // Release native image buffers
        ReleaseDC(h_wnd_screen, h_dc_screen); // don't need screen anymore
        DeleteDC(h_dc);
        DeleteObject(h_bmp);

        Ok(Screenshot {
            data,
            data_r_and_b_switched: data_color_invert,
            height: height as usize,
            width: width as usize,
            row_len: width as usize * PIXEL_WIDTH,
        })
    }
}

#[test]
fn test_get_screenshot() {
    let s: Screenshot = get_screenshot().unwrap();
    println!(
        "width: {}\nheight: {}\nbytes: {}",
        s.width,
        s.height,
        s.len()
    );
}
