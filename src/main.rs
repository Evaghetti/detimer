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

impl Timer {
    fn is_zero(&self) -> bool {
        self.seconds == 0 && self.minutes == 0
    }
}

struct TimerNotifySound {
    sink: Sink,
    _stream: OutputStream,
    path: String,
}

impl TimerNotifySound {
    fn load(path: &str) -> Result<(Sink, OutputStream), &'static str> {
        let (_stream, stream_handle) = if let Ok(args) = OutputStream::try_default() {
            args
        } else {
            return Err("Erro ao criar canal de áudio");
        };
        let source = if let Ok(file) = File::open(path) {
            if let Ok(final_decoder) = Decoder::new(file) {
                final_decoder
            } else {
                return Err("Erro ao decodificar arquivo de som, talvez o formato não seja válido");
            }
        } else {
            return Err("Erro abrindo arquivo de som");
        };

        let sink = if let Ok(sink) = Sink::try_new(&stream_handle) {
            sink
        } else {
            return Err("Erro ao inicializar player");
        };

        sink.append(source);
        sink.pause();
        Ok((sink, _stream))
    }

    fn new(path: &str) -> Result<Self, &'static str> {
        let (sink, _stream) = Self::load(path)?;

        Ok(Self {
            sink,
            _stream,
            path: path.to_string(),
        })
    }

    fn play(&self) -> Result<Self, &'static str> {
        self.sink.play();
        self.sink.sleep_until_end();

        Self::new(&self.path)
    }
}

#[derive(Parser)]
#[command(name = "detimer", about = "Contador pras livezinhas do Ronan :D")]
struct TimerConfig {
    /// Quanto tempo em segundos deve contar
    #[arg(
        short = 's',
        long = "segundos",
        conflicts_with = "time_minutes",
        required = true
    )]
    time_seconds: Option<i32>,
    /// Quanto tempo em minutos deve contar
    #[arg(
        short = 'm',
        long = "minutos",
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

    /// Quantas sprints serão feitas
    #[arg(short = 'p', long = "sprint", requires = "interval_time")]
    sprint_count: Option<u32>,

    /// Tempo que dura o intervalo entre sprints
    #[arg(
        short = 'i',
        long = "interval-time",
        requires = "out_interval",
        requires = "sprint_count"
    )]
    interval_time: Option<u32>,

    /// Onde informar se timer atual é sprint ou intervalo
    #[arg(short = 'P', long = "out-interval")]
    out_interval: Option<String>,
}

fn write(out: Option<&str>, content: &str) -> Result<(), Error> {
    let mut writer: Box<dyn Write> = if let Some(ref path) = out {
        Box::new(File::create(path)?)
    } else {
        Box::new(std::io::stdout())
    };

    writeln!(writer, "{}", content)?;

    Ok(())
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

    fn get_time_interval(&mut self) -> Timer {
        if let Some(current_sprint) = self.sprint_count {
            self.sprint_count = current_sprint.checked_sub(1);
        }

        Timer {
            seconds: 0,
            minutes: self.interval_time.unwrap_or_else(|| {
                println!("Quanto tempo de intervalo?");
                loop {
                    let mut inputted_data = String::new();
                    std::io::stdin()
                        .read_line(&mut inputted_data)
                        .expect("Não foi possível ler de stdin");
                    if let Ok(num) = inputted_data.trim().parse() {
                        return num;
                    }
                }
            }) as i32,
        }
    }

    fn run_timer(&self, mut timer: Timer) -> Result<(), Error> {
        let mut last_time = Instant::now();

        write(
            self.out.as_deref(),
            format!("{:0>2}:{:0>2}", timer.minutes, timer.seconds).as_str(),
        )?;

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

                write(
                    self.out.as_deref(),
                    format!("{:0>2}:{:0>2}", timer.minutes, timer.seconds).as_str(),
                )?;
                last_time = now;
            }
        }
    }
}

fn run_timer(
    config: &TimerConfig,
    sound: Option<&mut TimerNotifySound>,
    timer: Timer,
) -> Result<(), &'static str> {
    match config.run_timer(timer) {
        Ok(_) => {
            if let Some(sound) = sound {
                *sound = sound.play()?;
            }
            Ok(())
        }
        Err(_) => Err("Erro ao escrever em arquivo"),
    }
}

fn main() -> Result<(), &'static str> {
    let mut config = TimerConfig::parse();
    let mut sound = if let Some(ref path) = config.notify_sound {
        Some(TimerNotifySound::new(path)?)
    } else {
        None
    };

    if config.out_interval.is_none() {
        run_timer(&config, sound.as_mut(), config.get_time()?)
    } else {
        let mut current_sprint = 1;
        if let Some(max_sprint) = config.sprint_count {
            config.sprint_count = max_sprint.checked_sub(1);

            loop {
                let timer = config.get_time()?;
                write(
                    config.out_interval.as_deref(),
                    &format!("Sprint {}/{}", current_sprint, max_sprint),
                )
                .expect("Erro escrevendo para output");
                run_timer(&config, sound.as_mut(), timer)?;
                if current_sprint == max_sprint {
                    break;
                }

                let time_interval = config.get_time_interval();
                if time_interval.is_zero() {
                    break;
                }
                write(
                    config.out_interval.as_deref(),
                    &format!("Intervalo {}/{}", current_sprint, max_sprint),
                )
                .expect("Erro escrevendo para output");
                run_timer(&config, sound.as_mut(), time_interval)?;

                current_sprint += 1;
            }
        } else {
            loop {
                let timer = config.get_time()?;
                write(
                    config.out_interval.as_deref(),
                    &format!("Sprint {}", current_sprint),
                )
                .expect("Erro escrevendo para output");
                run_timer(&config, sound.as_mut(), timer)?;

                let time_interval = config.get_time_interval();
                if time_interval.is_zero() {
                    break;
                }
                write(
                    config.out_interval.as_deref(),
                    &format!("Intervalo {}", current_sprint),
                )
                .expect("Erro escrevendo para output");
                run_timer(&config, sound.as_mut(), time_interval)?;

                println!("Aperte enter para iniciar próxima sprint");
                let mut inputted_data = String::new();
                std::io::stdin()
                    .read_line(&mut inputted_data)
                    .expect("Não foi possível ler de stdin");

                current_sprint += 1;
            }
        }

        Ok(())
    }
}
