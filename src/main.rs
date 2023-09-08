use std::{
    fs::File,
    io::{Error, Write},
    time::Instant,
};

use clap::Parser;
use rodio::{Decoder, OutputStream, Sink};

struct Timer {
    seconds: i32,
    minutes: i32,
}

struct TimerNotifySound {
    sink: Sink,
    _stream: OutputStream,
}

impl TimerNotifySound {
    fn new(path: &str) -> Result<Self, &'static str> {
        let source = if let Ok(file) = File::open(path) {
            if let Ok(final_decoder) = Decoder::new(file) {
                final_decoder
            } else {
                return Err("Erro ao decodificar arquivo de som, talvez o formato não seja válido");
            }
        } else {
            return Err("Erro abrindo arquivo de som");
        };

        let (_stream, stream_handle) = if let Ok(args) = OutputStream::try_default() {
            args
        } else {
            return Err("Erro ao criar canal de áudio");
        };

        let sink = if let Ok(sink) = Sink::try_new(&stream_handle) {
            sink
        } else {
            return Err("Erro ao inicializar player");
        };

        let sink = sink;
        sink.append(source);
        sink.pause();

        Ok(Self { sink, _stream })
    }

    fn play(&self) {
        self.sink.play();
        self.sink.sleep_until_end();
    }
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
    #[arg(short = 'o', long = "out")]
    out: Option<String>,

    /// Áudio que irá tocar ao terminar o timer programado
    #[arg(short = 'n', long = "notify-sound")]
    notify_sound: Option<String>,
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
        let mut writer: Box<dyn Write> = if let Some(path) = self.out.as_ref() {
            Box::new(File::create(path)?)
        } else {
            Box::new(std::io::stdout())
        };

        writeln!(writer, "{}", content)?;

        Ok(())
    }

    fn run_timer(&self, mut timer: Timer) -> Result<(), Error> {
        let mut last_time = Instant::now();

        self.write(format!("{:0>2}:{:0>2}", timer.minutes, timer.seconds).as_str())?;

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

                self.write(format!("{:0>2}:{:0>2}", timer.minutes, timer.seconds).as_str())?;
                last_time = now;
            }
        }
    }
}

fn main() -> Result<(), &'static str> {
    let config = TimerConfig::parse();
    let timer = config.get_time()?;
    let sound = if let Some(ref path) = config.notify_sound {
        Some(TimerNotifySound::new(path)?)
    } else {
        None
    };

    match config.run_timer(timer) {
        Ok(_) => {
            if let Some(sound) = sound {
                sound.play();
            }
            Ok(())
        }
        Err(_) => Err("Erro ao escrever em arquivo"),
    }
}
