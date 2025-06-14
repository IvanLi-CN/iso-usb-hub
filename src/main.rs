// src/main.rs
#![no_std]
#![no_main]

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice as EmbassySpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{self, I2c}; // Import i2c module, I2c struct
use embassy_stm32::spi::{Config as SpiConfig, Spi as Stm32Spi};
use embassy_stm32::time::{Hertz, khz}; // Import khz and Hertz
use embassy_stm32::{bind_interrupts, mode, peripherals}; // Import bind_interrupts, mode, peripherals
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_alloc::LlffHeap as Heap;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::WebColors;
use gc9d01::{Config as DisplayDriverConfig, GC9D01, Orientation, Timer as Gc9d01Timer};
use static_cell::StaticCell;

use core::ptr;
use {defmt_rtt as _, panic_probe as _};

// Add imports for INA226 and shared bus I2C device
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice as EmbassyI2cDevice; // Alias for clarity
use ina226::INA226;
// Removed unused imports: AsyncI2c

use defmt::*;
use display::dashboard::Dashboard;
mod display;

extern crate alloc;

#[global_allocator]
static HEAP: Heap = Heap::empty();

// This marks the entrypoint of our application and binds interrupts.
bind_interrupts!(
    struct Irqs {
        I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
        I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting GC9D01 Example");

    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi48 = Some(Hsi48Config {
            sync_from_usb: true,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL85,
            divp: None,
            divq: None,
            // Main system clock at 170 MHz
            divr: Some(PllRDiv::DIV2),
        });
        config.rcc.mux.adc12sel = mux::Adcsel::SYS;
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.mux.clk48sel = mux::Clk48sel::HSI48;
        // config.enable_ucpd1_dead_battery = true;
    }
    let p = embassy_stm32::init(config);

    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 8192;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(ptr::addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    // Initialize I2C1
    let i2c_scl = p.PA15; // SCL pin for I2C1
    let i2c_sda = p.PB7; // SDA pin for I2C1

    let mut i2c_config = i2c::Config::default(); // Use full path for Config
    // Configure I2C speed if needed, default is 100kHz
    // i2c_config.speed = embassy_stm32::i2c::Speed::Standard; // Or Fast, FastPlus

    // Initialize I2C1 with correct parameter order
    let i2c1 = I2c::new(
        p.I2C1,     // Instance
        i2c_scl,    // SCL pin
        i2c_sda,    // SDA pin
        Irqs,       // Interrupts struct
        p.DMA1_CH2, // RX DMA
        p.DMA1_CH3, // TX DMA
        khz(100),   // Frequency
        i2c_config, // Config parameter
    );

    // Create a static mutex for the I2C bus using the full I2c type
    // Create a static mutex for the I2C bus using the full I2c type
    static I2C1_BUS_CELL: StaticCell<Mutex<CriticalSectionRawMutex, I2c<'static, mode::Async>>> =
        StaticCell::new(); // Use full I2c type
    let i2c1_bus_mutex_ref = I2C1_BUS_CELL.init(Mutex::new(i2c1));

    // Initialize INA226 sensors using I2cDevice for shared bus access
    // Create I2cDevice instances from the shared bus mutex with correct type parameters
    let i2c_device_1 = EmbassyI2cDevice::new(i2c1_bus_mutex_ref);
    let i2c_device_2 = EmbassyI2cDevice::new(i2c1_bus_mutex_ref);
    let i2c_device_3 = EmbassyI2cDevice::new(i2c1_bus_mutex_ref);

    // Initialize INA226 sensors with the I2cDevice instances
    let mut ina226_1 = INA226::new(i2c_device_1, 0x40);
    let mut ina226_2 = INA226::new(i2c_device_2, 0x41);
    let mut ina226_3 = INA226::new(i2c_device_3, 0x44);

    // Configure INA226 sensors (optional, default config is often fine)
    // Example: Set calibration register for current/power readings
    ina226_1.callibrate(0.005, 4.0).await.unwrap();
    ina226_2.callibrate(0.010, 4.0).await.unwrap();
    ina226_3.callibrate(0.010, 4.0).await.unwrap();

    info!("INA226 sensors initialized.");

    struct EmbassyDisplayTimer;
    impl Gc9d01Timer for EmbassyDisplayTimer {
        async fn after_millis(milliseconds: u64) {
            embassy_time::Timer::after_millis(milliseconds).await;
        }
    }

    let spi_peripheral_instance = p.SPI1;
    impl embedded_hal::digital::ErrorType for EmbassyDisplayTimer {
        type Error = core::convert::Infallible;
    }
    let sck_pin = p.PB3;
    let mosi_pin = p.PA7;

    // According to compiler error E0107 (note), Output<'d> has 0 type generic arguments.
    // This contradicts embassy-stm32 source, but we follow the compiler error.
    let cs_pin_output = Output::new(p.PA4, Level::High, Speed::VeryHigh);

    let dc_pin = Output::new(p.PB0, Level::Low, Speed::VeryHigh);
    let rst_pin = Output::new(p.PC4, Level::Low, Speed::VeryHigh);

    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(48_000_000);

    let spi_bus = Stm32Spi::new_txonly(
        spi_peripheral_instance,
        sck_pin,
        mosi_pin,
        p.DMA1_CH1,
        spi_config,
    );

    // According to compiler error E0107 (note), Spi<'d, M: PeriMode> has 1 type generic argument M.
    // For async SPI1, M should be (peripherals::SPI1, mode::Async).

    static SPI_BUS_CELL: StaticCell<
        Mutex<CriticalSectionRawMutex, Stm32Spi<'static, mode::Async>>,
    > = StaticCell::new();
    let spi_bus_mutex_ref = SPI_BUS_CELL.init(Mutex::new(spi_bus));

    // EmbassySpiDevice<'a, Mtx: RawMutex, BUS: SpiBus, CS: OutputPin>
    // CS type is now CsPinConcreteType = Output<'static>
    let spi_device = EmbassySpiDevice::<
        'static,
        CriticalSectionRawMutex,
        Stm32Spi<'static, mode::Async>,
        Output<'static>,
    >::new(spi_bus_mutex_ref, cs_pin_output);

    let display_config = DisplayDriverConfig {
        width: 160,
        height: 40,
        orientation: Orientation::PortraitSwapped,
        rgb: false,
        inverted: false,
        dx: 0,
        dy: 0,
    };

    static DISPLAY_BUFFER_CELL: StaticCell<[u8; gc9d01::BUF_SIZE]> = StaticCell::new();
    let buffer_slice: &mut [u8] = DISPLAY_BUFFER_CELL.init([0; gc9d01::BUF_SIZE]);

    let mut display: GC9D01<
        '_,
        EmbassySpiDevice<
            'static,
            CriticalSectionRawMutex,
            Stm32Spi<'static, mode::Async>,
            Output<'static>,
        >,
        Output<'_>,
        Output<'_>,
        EmbassyDisplayTimer,
    > = GC9D01::new(display_config, spi_device, dc_pin, rst_pin, buffer_slice);

    info!("Initializing display...");
    match display.init().await {
        Ok(_) => info!("Display initialized successfully!"),
        Err(e) => error!("Display initialization failed: {:?}", e),
    }
    info!("Display initialization complete."); // Added log

    // Instantiate Dashboard
    let mut dashboard = Dashboard::new();

    display.fill_color(Rgb565::CSS_BLACK).await.unwrap();

    info!("Drawing test pattern.");
    let colors = [
        Rgb565::CSS_WHITE,
        Rgb565::CSS_YELLOW,
        Rgb565::CSS_CYAN,
        Rgb565::CSS_GREEN,
        Rgb565::CSS_MAGENTA,
        Rgb565::CSS_RED,
        Rgb565::CSS_BLUE,
        Rgb565::CSS_BLACK,
    ];

    // Each stripe is 5 pixels wide and 160 pixels high
    const STRIPE_WIDTH: u16 = 20;
    const STRIPE_HEIGHT: u16 = 40;

    // Create a buffer for one stripe's pixel data
    let mut stripe_pixels = [Rgb565::CSS_BLACK; (STRIPE_WIDTH * STRIPE_HEIGHT) as usize];

    for (i, color) in colors.iter().enumerate() {
        let x = i as u16 * STRIPE_WIDTH;

        // Fill the stripe buffer with the current color
        for pixel in stripe_pixels.iter_mut() {
            *pixel = *color;
        }

        // Write the pixel data for the current stripe
        display
            .write_area(x, 0, STRIPE_WIDTH, STRIPE_HEIGHT, &stripe_pixels)
            .await
            .unwrap();
    }

    // Initial delay before starting the loop
    embassy_time::Timer::after_secs(1).await;

    loop {
        // Read data from INA226 sensors
        // Use correct async function names and handle Option<f64> return types
        let voltage1 = ina226_1.bus_voltage_millivolts().await.unwrap_or(0.0);
        let current1 = ina226_1.current_amps().await.unwrap_or(None).unwrap_or(0.0);
        let power1 = ina226_1.power_watts().await.unwrap_or(None).unwrap_or(0.0);

        let voltage2 = ina226_2.bus_voltage_millivolts().await.unwrap_or(0.0);
        let current2 = ina226_2.current_amps().await.unwrap_or(None).unwrap_or(0.0);
        let power2 = ina226_2.power_watts().await.unwrap_or(None).unwrap_or(0.0);

        let voltage3 = ina226_3.bus_voltage_millivolts().await.unwrap_or(0.0);
        let current3 = ina226_3.current_amps().await.unwrap_or(None).unwrap_or(0.0);
        let power3 = ina226_3.power_watts().await.unwrap_or(None).unwrap_or(0.0);

        // Prepare data for Dashboard, converting f64 to f32
        let sensor_data = [
            ((voltage1 / 1000.0) as f32, current1 as f32, power1 as f32),
            ((voltage2 / 1000.0) as f32, current2 as f32, power2 as f32),
            ((voltage3 / 1000.0) as f32, current3 as f32, power3 as f32),
        ];

        // Update Dashboard data
        dashboard.update_data(sensor_data);

        // Draw Dashboard directly to the display
        dashboard.draw(&mut display).await.unwrap();

        // Wait for 1 second before the next update
        embassy_time::Timer::after_millis(100).await;
    }
}
