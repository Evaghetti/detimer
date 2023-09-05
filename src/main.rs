use std::{
    fs::File,
    io::{Error, Write},
    time::Instant,
};

use clap::Parser;

struct Timer {
    seconds: i32,
    minutes: i32,
}

#[derive(Parser)]
#[command(name = "detimer", about = "Contador pras livezinhas do Ronan :D")]
struct TimerConfig {
    /// Quanto tempo em segundos deve contar
    #[arg(
        short = 's',
        long = "segundos",
        group = "timer_seconds",
        conflicts_with = "time_minutes",
        required = true
    )]
    time_seconds: Option<i32>,
    /// Quanto tempo em minutos deve contar
    #[arg(
        short = 'm',
        long = "minutos",
        group = "timer_minutes",
        conflicts_with = "time_seconds",
        required = true
    )]
    time_minutes: Option<i32>,
    /// Pra onde escrever (por padrão stdout)
    #[arg(short = 'o', long = "output")]
    output: Option<String>,
}

impl TimerConfig {
    fn get_time(&self) -> Result<Timer, &'static str> {
        let actual_seconds = if let Some(time) = self.time_minutes {
            let time = time * 60;

            Some(time)
        } else {
            self.time_seconds
        };

        if let Some(time) = actual_seconds {
            let seconds = time % 60;
            let minutes = time / 60;

            Ok(Timer { seconds, minutes })
        } else {
            Err("Tem que me dizer um time maninho")
        }
    }

    fn write(&self, content: &str) -> Result<(), Error> {
        let mut writer: Box<dyn Write> = if let Some(path) = self.output.as_ref() {
            Box::new(File::create(path)?)
        } else {
            Box::new(std::io::stdout())
        };

        writeln!(writer, "{}", content)?;

        Ok(())
    }
}

fn run_timer(out: TimerConfig, mut timer: Timer) -> Result<(), Error> {
    let mut last_time = Instant::now();

    out.write(format!("{:0>2}:{:0>2}", timer.minutes, timer.seconds).as_str())?;

    loop {
        let now = Instant::now();
        let passed_time = (now - last_time).as_secs() as i32;

        if passed_time >= 1 {
            timer.seconds -= passed_time;
            if timer.seconds < 0 {
                let difference = timer.seconds.abs();
                timer.seconds = 60 - difference;

                timer.minutes -= 1;

                if timer.minutes < 0 {
                    return Ok(());
                }
            }

            out.write(format!("{:0>2}:{:0>2}", timer.minutes, timer.seconds).as_str())?;
            last_time = now;
        }
    }
}

fn main() -> Result<(), &'static str> {
    let config = TimerConfig::parse();
    let timer = config.get_time()?;

    match run_timer(config, timer) {
        Ok(_) => Ok(()),
        Err(_) => Err("Erro ao escrever em arquivo"),
    }
}
