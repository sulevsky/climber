use core::fmt::Write;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::UartTx;
use heapless::String;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use rtt_target::rprintln;

struct Logger;

static mut USART_TX: Option<UartTx<'static, Async>> = None;

pub fn init(uart_tx: Option<UartTx<'static, Async>>) -> Result<(), SetLoggerError> {
    if let Some(tx) = uart_tx {
        unsafe { USART_TX = Some(tx) };
    }
    log::set_logger(&Logger)?;
    log::set_max_level(LevelFilter::Info);
    Ok(())
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            rprintln!("{} - {}", record.level(), record.args());
            critical_section::with(|_| {
                if let Some(tx) = unsafe { (*&raw mut USART_TX).as_mut() } {
                    let mut line: String<128> = String::new();
                    if writeln!(line, "[{}] {}", record.level(), record.args()).is_ok() {
                        let _ = tx.blocking_write(line.as_bytes());
                    }
                }
            });
        }
    }

    fn flush(&self) {}
}
