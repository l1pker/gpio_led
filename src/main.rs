use std::thread::sleep;
use std::time::Duration;
use chrono::Local;
use std::fs::File;
use std::io::Write;

#[cfg(target_arch = "arm")]
use rppal::gpio::{Gpio, Level, InputPin, OutputPin};

#[cfg(not(target_arch = "arm"))]
mod gpio_mock {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Level {
        Low,
        High,
    }

    pub struct InputPin {
        level: Arc<Mutex<Level>>,
    }

    impl InputPin {
        pub fn new() -> Self {
            let level = Arc::new(Mutex::new(Level::High));
            let level_clone = level.clone();

            thread::spawn(move || {
                loop {
                    {
                        let mut l = level_clone.lock().unwrap();
                        *l = if *l == Level::High { Level::Low } else { Level::High };
                    }
                    thread::sleep(Duration::from_secs(3)); // Імітація натискання кнопки
                }
            });

            Self { level }
        }

        pub fn read(&self) -> Level {
            *self.level.lock().unwrap()
        }

        pub fn is_low(&self) -> bool {
            self.read() == Level::Low
        }
    }

    pub struct OutputPin {
        level: Arc<Mutex<Level>>,
    }

    impl OutputPin {
        pub fn new() -> Self {
            Self { level: Arc::new(Mutex::new(Level::Low)) }
        }

        pub fn write(&self, level: Level) {
            let mut l = self.level.lock().unwrap();
            *l = level;
        }
    }
}
#[cfg(not(target_arch = "arm"))]
use gpio_mock::{InputPin, OutputPin, Level};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let button_pin_num = 17;
    let led_pin_num = 18;

    #[cfg(target_arch = "arm")]
    let gpio = Gpio::new().map_err(|e| {
        log_error(&format!("Помилка ініціалізації GPIO: {}", e)).unwrap();
        e
    })?;

    #[cfg(target_arch = "arm")]
    let mut button_pin: InputPin = gpio.get(button_pin_num)?.into_input_pullup();

    #[cfg(target_arch = "arm")]
    let mut led_pin: OutputPin = gpio.get(led_pin_num)?.into_output();

    #[cfg(not(target_arch = "arm"))]
    let mut button_pin = InputPin::new();

    #[cfg(not(target_arch = "arm"))]
    let mut led_pin = OutputPin::new();

    let mut led_state = Level::Low;
    let mut last_button_state = button_pin.read();

    let mut file = File::create("log.txt").map_err(|e| {
        log_error(&format!("Помилка створення файлу логу: {}", e)).unwrap();
        e
    })?;

    loop {
        let button_state = button_pin.read();
        let current_time = Local::now();

        if button_state != last_button_state {
            let log_message = format!("[{}] Стан кнопки: {:?}\n", current_time, button_state);
            if let Err(e) = file.write_all(log_message.as_bytes()) {
                log_error(&format!("Помилка запису в файл: {}", e))?;
                return Err(Box::new(e));
            }
            last_button_state = button_state;

            if button_state == Level::Low {
                led_state = match led_state {
                    Level::Low => Level::High,
                    Level::High => Level::Low,
                };
                led_pin.write(led_state);
                let log_message = format!("[{}] Стан світлодіода змінено: {:?}\n", current_time, led_state);
                if let Err(e) = file.write_all(log_message.as_bytes()) {
                    log_error(&format!("Помилка запису в файл: {}", e))?;
                    return Err(Box::new(e));
                }

                while button_pin.is_low() {
                    sleep(Duration::from_millis(10));
                }
            }
        }
        sleep(Duration::from_millis(50));
    }
}

fn log_error(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let current_time = Local::now();
    let log_message = format!("[{}] Помилка: {}\n", current_time, message);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("log.txt")?;
    file.write_all(log_message.as_bytes())?;
    Ok(())
}
