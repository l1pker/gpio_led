use std::env;
#[cfg(target_os = "linux")]
use rppal::gpio::{Gpio, Level};
use std::thread::sleep;
use std::time::Duration;
use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Визначаємо платформу
    let is_rpi = detect_raspberry_pi();

    println!("Running on: {}", if is_rpi { "Raspberry Pi" } else { "Other platform" });

    let button_pin_num = 17;
    let led_pin_num = 18;
    
    #[cfg(target_os = "linux")]
    let gpio = Gpio::new().expect("Помилка ініціалізації GPIO");

    #[cfg(target_os = "linux")]
    let mut button_pin = gpio.get(button_pin_num)?.into_input_pullup();
    
    #[cfg(target_os = "linux")]
    let mut led_pin = gpio.get(led_pin_num)?.into_output();

    let mut led_state = Level::Low;
    let mut last_button_state = if is_rpi { button_pin.read() } else { Level::High };

    let mut file = OpenOptions::new().write(true).append(true).create(true).open("log.txt")?;

    loop {
        let button_state = if is_rpi { button_pin.read() } else { Level::High };
        let current_time = Local::now();

        if button_state != last_button_state {
            let log_message = format!("[{}] Стан кнопки: {:?}\n", current_time, button_state);
            file.write_all(log_message.as_bytes())?;
            last_button_state = button_state;

            if button_state == Level::Low {
                led_state = match led_state {
                    Level::Low => Level::High,
                    Level::High => Level::Low,
                };
                
                #[cfg(target_os = "linux")]
                led_pin.write(led_state);

                let log_message = format!("[{}] Стан світлодіода змінено: {:?}\n", current_time, led_state);
                file.write_all(log_message.as_bytes())?;

                while is_rpi && button_pin.is_low() {
                    sleep(Duration::from_millis(10));
                }
            }
        }
        sleep(Duration::from_millis(50));
    }
}

/// Функція для визначення, чи працюємо ми на Raspberry Pi
fn detect_raspberry_pi() -> bool {
    if cfg!(target_os = "linux") {
        std::path::Path::new("/proc/device-tree/model").exists()
    } else {
        false
    }
}
