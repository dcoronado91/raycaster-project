use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::f32::consts::PI;
use std::time::Duration;

const SAMPLE_RATE: u32 = 44_100;

const MUSIC_DIR: &str = "assets/audio";
const MUSIC_EXTENSIONS: &[&str] = &["mp3", "ogg", "wav", "flac", "m4a", "aac"];

/// Busca el primer archivo de audio soportado dentro de `MUSIC_DIR`, sin
/// importar como se llame: asi no hay que acertarle a un nombre exacto,
/// alcanza con poner ahi el archivo de musica.
fn find_music_file() -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(MUSIC_DIR).ok()?;
    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| MUSIC_EXTENSIONS.iter().any(|candidate| candidate.eq_ignore_ascii_case(ext)))
        })
}

#[derive(Clone, Copy)]
enum Waveform {
    Sine,
    Square,
}

/// Un tono sintetizado (sin depender de ningun archivo de sonido): barre de
/// `start_freq` a `end_freq` a lo largo de `duration_secs`, con un fundido
/// de salida para que no truene al terminar. Es la base de todos los
/// efectos de sonido del juego.
struct Tone {
    waveform: Waveform,
    total_samples: usize,
    sample_index: usize,
    start_freq: f32,
    end_freq: f32,
    volume: f32,
}

impl Tone {
    fn new(waveform: Waveform, duration_secs: f32, start_freq: f32, end_freq: f32, volume: f32) -> Self {
        Tone {
            waveform,
            total_samples: (duration_secs * SAMPLE_RATE as f32) as usize,
            sample_index: 0,
            start_freq,
            end_freq,
            volume,
        }
    }

    fn silence(duration_secs: f32) -> Self {
        Tone::new(Waveform::Sine, duration_secs, 1.0, 1.0, 0.0)
    }
}

impl Iterator for Tone {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.sample_index >= self.total_samples || self.total_samples == 0 {
            return None;
        }

        let t = self.sample_index as f32 / self.total_samples as f32;
        let freq = self.start_freq + (self.end_freq - self.start_freq) * t;
        let time = self.sample_index as f32 / SAMPLE_RATE as f32;
        let phase = 2.0 * PI * freq * time;

        let raw = match self.waveform {
            Waveform::Sine => phase.sin(),
            Waveform::Square => {
                if phase.sin() >= 0.0 {
                    1.0
                } else {
                    -1.0
                }
            }
        };

        let envelope = (1.0 - t).powf(1.5); // fundido de salida
        self.sample_index += 1;
        Some(raw * self.volume * envelope)
    }
}

impl Source for Tone {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(self.total_samples as f32 / SAMPLE_RATE as f32))
    }
}

/// Varios `Tone` reproducidos uno tras otro (por ejemplo, los dos clics de
/// recargar o las notas de un arpegio).
struct ToneSequence {
    tones: std::collections::VecDeque<Tone>,
}

impl ToneSequence {
    fn new(tones: Vec<Tone>) -> Self {
        ToneSequence { tones: tones.into() }
    }
}

impl Iterator for ToneSequence {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        loop {
            let sample = self.tones.front_mut()?.next();
            if sample.is_some() {
                return sample;
            }
            self.tones.pop_front();
        }
    }
}

impl Source for ToneSequence {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Maneja la salida de audio del juego: musica de fondo en loop (desde un
/// archivo) y efectos de sonido sintetizados en codigo. Si no hay
/// dispositivo de audio disponible, `Audio::new` devuelve `None` y el resto
/// del juego sigue funcionando en silencio (nunca crashea por esto).
pub struct Audio {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    music_sink: Option<Sink>,
}

impl Audio {
    pub fn new() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        Some(Audio {
            _stream: stream,
            handle,
            music_sink: None,
        })
    }

    /// Busca un archivo de musica de fondo en `MUSIC_CANDIDATES` y lo pone
    /// en loop. Si no encuentra ninguno (o no se puede decodificar), avisa
    /// por consola y el juego sigue sin musica.
    pub fn play_music(&mut self) {
        let Some(path) = find_music_file() else {
            eprintln!("No se encontro musica de fondo en {MUSIC_DIR}/ (mp3, ogg, wav, flac, m4a o aac).");
            return;
        };

        let Ok(file) = std::fs::File::open(&path) else {
            return;
        };
        let Ok(source) = rodio::Decoder::new(std::io::BufReader::new(file)) else {
            eprintln!("No se pudo decodificar {}; el juego sigue sin musica.", path.display());
            return;
        };

        if let Ok(sink) = Sink::try_new(&self.handle) {
            sink.set_volume(0.35);
            sink.append(source.repeat_infinite());
            self.music_sink = Some(sink);
        }
    }

    fn play_one_shot(&self, source: impl Source<Item = f32> + Send + 'static) {
        let _ = self.handle.play_raw(source);
    }

    /// Destello agudo y descendente, tipo "laser" digital.
    pub fn play_shoot(&self) {
        self.play_one_shot(Tone::new(Waveform::Square, 0.12, 900.0, 280.0, 0.35));
    }

    /// Un Agente eliminado: "glitch" mas largo y agudo que el disparo.
    pub fn play_agent_hit(&self) {
        self.play_one_shot(Tone::new(Waveform::Square, 0.2, 1300.0, 200.0, 0.3));
    }

    /// Dos clics cortos, como sacar y meter un cargador.
    pub fn play_reload(&self) {
        self.play_one_shot(ToneSequence::new(vec![
            Tone::new(Waveform::Square, 0.05, 260.0, 260.0, 0.25),
            Tone::silence(0.06),
            Tone::new(Waveform::Square, 0.05, 220.0, 220.0, 0.25),
        ]));
    }

    /// Tono grave y largo, para el Game Over.
    pub fn play_game_over(&self) {
        self.play_one_shot(Tone::new(Waveform::Sine, 0.9, 320.0, 40.0, 0.45));
    }

    /// Arpegio ascendente de 3 notas, para nivel completado.
    pub fn play_success(&self) {
        self.play_one_shot(ToneSequence::new(vec![
            Tone::new(Waveform::Sine, 0.14, 440.0, 440.0, 0.3),
            Tone::new(Waveform::Sine, 0.14, 554.0, 554.0, 0.3),
            Tone::new(Waveform::Sine, 0.22, 660.0, 660.0, 0.32),
        ]));
    }

    /// Blip corto al mover la seleccion en el menu.
    pub fn play_menu_blip(&self) {
        self.play_one_shot(Tone::new(Waveform::Sine, 0.06, 600.0, 600.0, 0.2));
    }
}
