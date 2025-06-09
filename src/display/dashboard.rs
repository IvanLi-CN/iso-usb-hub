// src/display/dashboard.rs
// Dashboard 页面模块

// Keep Rgb565 for colors
use embedded_graphics::pixelcolor::Rgb565;
// Remove other embedded_graphics imports as drawing primitives won't be used directly
use embedded_graphics::prelude::RgbColor;
// Removed: use embedded_graphics::prelude::*; // Unused import
// use embedded_graphics::{
//     mono_font::{ascii::FONT_6X10, MonoTextStyle},
//     prelude::*,
//     text::{Alignment, Text},
//     geometry::Point,
// };

use gc9d01::GC9D01; // Import GC9D01
use crate::display::font::{char_to_mono_bitmap, mono_bitmap_to_rgb565, FONT_8X12_WIDTH, FONT_8X12_HEIGHT}; // Updated constant names

// Import necessary traits for GC9D01 (these are bounds on the GC9D01 struct itself)
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiDevice;
use gc9d01::Timer as Gc9d01Timer; // Moved this import up
use core::convert::TryInto; // Added import for try_into
use alloc::format; // Added import for alloc::format!

#[derive(Debug)]
pub enum Error {
    // Add specific error types later if needed
    DriverError, // Placeholder for errors from the GC9D01 driver
    // Add other errors like FontError, LayoutError, etc. as needed
}


// Define colors
const COLOR_VOLTAGE: Rgb565 = Rgb565::YELLOW;
const COLOR_CURRENT: Rgb565 = Rgb565::RED;
const COLOR_POWER: Rgb565 = Rgb565::GREEN;

// Dashboard struct, contains data to display
pub struct Dashboard {
    voltage_v: f32,
    power_w1: f32,
    power_w2: f32,
    current_a1: f32,
    current_a2: f32,
    current_a3: f32,
}

impl Dashboard {
    // Create new Dashboard instance
    pub fn new() -> Self {
        Self {
            voltage_v: 0.0,
            power_w1: 0.0,
            power_w2: 0.0,
            current_a1: 0.0,
            current_a2: 0.0,
            current_a3: 0.0,
        }
    }

    // Update Dashboard display data
    // Pass in order V, W1, W2, A1, A2, A3
    pub fn update_data(&mut self, v: f32, w1: f32, w2: f32, a1: f32, a2: f32, a3: f32) {
        self.voltage_v = v;
        self.power_w1 = w1;
        self.power_w2 = w2;
        self.current_a1 = a1;
        self.current_a2 = a2;
        self.current_a3 = a3;
    }

    // Draw Dashboard directly to the display driver using write_area
    // Accept GC9D01 directly
    pub async fn draw<'a, BUS, DC, RST, TIMER>(&self, display: &mut GC9D01<'a, BUS, DC, RST, TIMER>) -> Result<(), Error>
    where
        BUS: SpiDevice,
        DC: OutputPin<Error = core::convert::Infallible>, // Specify Infallible error type
        RST: OutputPin<Error = core::convert::Infallible>, // Specify Infallible error type
        TIMER: Gc9d01Timer,
    {
        // Clear screen manually by writing black pixels to the whole area
        let screen_width = 160; // Assuming landscape 160x40
        let screen_height = 40;
        let _black_pixel = Rgb565::BLUE;
        // Create a buffer for a 20x20 block of black pixels
        // const BLOCK_SIZE: u16 = 20; // Removed unused constant

        let _ = display.fill_color(Rgb565::BLACK).await;
        // Handle potential remaining rows/columns if screen dimensions are not multiples of BLOCK_SIZE
        // (Assuming 160x40 is a multiple of 20x20, so no extra handling needed for this specific case)


        // Layout: 3 columns, 2 rows
        let col_width = screen_width / 3; // Approx 53
        let row_height = screen_height / 2; // 20

        // Buffer for character pixels (8x12)
        let mut char_pixel_buffer = [Rgb565::BLACK; FONT_8X12_WIDTH * FONT_8X12_HEIGHT]; // Updated constant names

        // Helper function to draw a string
        // Helper function to draw a string with right alignment
        async fn draw_string<'a, BUS, DC, RST, TIMER>(
            display: &mut GC9D01<'a, BUS, DC, RST, TIMER>,
            s: &str,
            right_edge_x: usize, // Right edge of the drawing area
            start_y: usize,
            fg_color: Rgb565,
            bg_color: Rgb565,
            char_pixel_buffer: &mut [Rgb565], // Pass buffer as argument
        ) -> Result<(), Error>
        where
            BUS: SpiDevice,
            DC: OutputPin<Error = core::convert::Infallible>, // Specify Infallible error type
            RST: OutputPin<Error = core::convert::Infallible>, // Specify Infallible error type
            TIMER: Gc9d01Timer,
        {
            let string_pixel_width = s.chars().count() * FONT_8X12_WIDTH;
            let start_x = right_edge_x.saturating_sub(string_pixel_width); // Calculate start_x for right alignment, handle potential underflow

            let mut current_x = start_x;
            for c in s.chars() {
                if let Some(bitmap) = char_to_mono_bitmap(c) {
                    mono_bitmap_to_rgb565(bitmap, fg_color, bg_color, char_pixel_buffer);

                    let x0 = current_x;
                    let y0 = start_y;
                    let _x1 = x0 + FONT_8X12_WIDTH - 1; // Updated constant name
                    let _y1 = y0 + FONT_8X12_HEIGHT - 1; // Updated constant name

                    display.write_area(
                        x0.try_into().unwrap(),
                        y0.try_into().unwrap(),
                        FONT_8X12_WIDTH.try_into().unwrap(), // Updated constant name
                        FONT_8X12_HEIGHT.try_into().unwrap(), // Updated constant name
                        char_pixel_buffer,
                    ).await.map_err(|_| Error::DriverError)?;

                    current_x += FONT_8X12_WIDTH; // Updated constant name
                } else {
                    // Handle characters not in font (e.g., draw a blank space)
                    current_x += FONT_8X12_WIDTH; // Updated constant name // Just advance cursor for now
                }
            }
            Ok(())
        }

        // Draw values
        let mut buffer: [u8; 10] = [0; 10]; // Buffer for float to string conversion

        // Row 1
        let voltage_str = self.float_to_string(&mut buffer, self.voltage_v); // Use self.float_to_string
        draw_string(display, &format!("{}V", voltage_str), col_width as usize, 0, COLOR_VOLTAGE, Rgb565::BLACK, &mut char_pixel_buffer).await?;

        let power1_str = self.float_to_string(&mut buffer, self.power_w1); // Use self.float_to_string
        draw_string(display, &format!("{}W", power1_str), (col_width * 2) as usize, 0, COLOR_POWER, Rgb565::BLACK, &mut char_pixel_buffer).await?;

        let power2_str = self.float_to_string(&mut buffer, self.power_w2); // Use self.float_to_string
        draw_string(display, &format!("{}W", power2_str), screen_width as usize, 0, COLOR_POWER, Rgb565::BLACK, &mut char_pixel_buffer).await?;

        // Row 2
        let current1_str = self.float_to_string(&mut buffer, self.current_a1); // Use self.float_to_string
        draw_string(display, &format!("{}A", current1_str), col_width as usize, row_height as usize, COLOR_CURRENT, Rgb565::BLACK, &mut char_pixel_buffer).await?;

        let current2_str = self.float_to_string(&mut buffer, self.current_a2); // Use self.float_to_string
        draw_string(display, &format!("{}A", current2_str), (col_width * 2) as usize, row_height as usize, COLOR_CURRENT, Rgb565::BLACK, &mut char_pixel_buffer).await?;

        let current3_str = self.float_to_string(&mut buffer, self.current_a3); // Use self.float_to_string
        draw_string(display, &format!("{}A", current3_str), screen_width as usize, row_height as usize, COLOR_CURRENT, Rgb565::BLACK, &mut char_pixel_buffer).await?;


        Ok(())
    }

    // Simplified float to string function (moved from inside draw)
    fn float_to_string<'a>(&self, buffer: &'a mut [u8], value: f32) -> &'a str { // Added &self and value parameter and lifetime
        // This is a very simplified implementation for demonstration only
        // Does not handle negative numbers, large numbers, or specific precision well
        let integer_part = value as i32;
        let decimal_part = ((value - integer_part as f32).abs() * 100.0) as i32; // Get two decimal places, handle negative input

        let mut cursor = 0;
        if value < 0.0 {
            buffer[cursor] = b'-';
            cursor += 1;
        }
        let mut temp = integer_part.abs();
        let mut divisor = 1;
        while divisor * 10 <= temp {
            divisor *= 10;
        }
        while divisor > 0 {
            buffer[cursor] = b'0' + (temp / divisor) as u8;
            cursor += 1;
            temp %= divisor;
            divisor /= 10;
        }
         if integer_part == 0 && value.abs() < 1.0 && value >= 0.0 { // Handle 0.x case
             buffer[cursor] = b'0';
             cursor += 1;
        } else if integer_part == 0 && value.abs() < 1.0 && value < 0.0 && cursor == 1 { // Handle -0.x case
             buffer[cursor] = b'0';
             cursor += 1;
        }


        buffer[cursor] = b'.';
        cursor += 1;

        buffer[cursor] = b'0' + (decimal_part / 10) as u8;
        cursor += 1;
        buffer[cursor] = b'0' + (decimal_part % 10) as u8;
        cursor += 1;

        core::str::from_utf8(&buffer[..cursor]).unwrap_or("Err")
    }
}