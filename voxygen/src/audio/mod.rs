pub mod channel;
pub mod fader;
pub mod music;
pub mod sfx;
pub mod soundcache;

use channel::{AudioType, Channel, ChannelTag};
use fader::Fader;
use soundcache::SoundCache;

use common::assets;
use cpal::traits::DeviceTrait;
use rodio::{Decoder, Device};
use vek::*;

const FALLOFF: f32 = 0.13;

pub struct AudioFrontend {
    pub device: String,
    pub device_list: Vec<String>,
    audio_device: Option<Device>,
    sound_cache: SoundCache,

    channels: Vec<Channel>,
    next_channel_id: usize,

    sfx_volume: f32,
    music_volume: f32,

    listener_pos: Vec3<f32>,
    listener_ori: Vec3<f32>,

    listener_ear_left: Vec3<f32>,
    listener_ear_right: Vec3<f32>,
}

impl AudioFrontend {
    /// Construct with given device
    pub fn new(device: String, channel_num: usize) -> Self {
        let mut channels = Vec::with_capacity(channel_num);
        let audio_device = get_device_raw(&device);
        if let Some(audio_device) = &audio_device {
            for _i in 0..channel_num {
                channels.push(Channel::new(&audio_device));
            }
        }
        Self {
            device: device.clone(),
            device_list: list_devices(),
            audio_device,
            sound_cache: SoundCache::new(),
            channels,
            next_channel_id: 1,
            sfx_volume: 1.0,
            music_volume: 1.0,
            listener_pos: Vec3::zero(),
            listener_ori: Vec3::zero(),
            listener_ear_left: Vec3::zero(),
            listener_ear_right: Vec3::zero(),
        }
    }

    /// Construct in `no-audio` mode for debugging
    pub fn no_audio() -> Self {
        Self {
            device: "none".to_string(),
            device_list: Vec::new(),
            audio_device: None,
            sound_cache: SoundCache::new(),
            channels: Vec::new(),
            next_channel_id: 1,
            sfx_volume: 1.0,
            music_volume: 1.0,
            listener_pos: Vec3::zero(),
            listener_ori: Vec3::zero(),
            listener_ear_left: Vec3::zero(),
            listener_ear_right: Vec3::zero(),
        }
    }

    /// Maintain audio
    pub fn maintain(&mut self, dt: f32) {
        for channel in self.channels.iter_mut() {
            channel.update(dt);
        }
    }

    pub fn get_channel(
        &mut self,
        audio_type: AudioType,
        channel_tag: Option<ChannelTag>,
    ) -> Option<&mut Channel> {
        if let Some(channel) = self.channels.iter_mut().find(|c| c.is_done()) {
            let id = self.next_channel_id;
            self.next_channel_id += 1;

            let volume = match audio_type {
                AudioType::Music => self.music_volume,
                _ => self.sfx_volume,
            };

            channel.set_id(id);
            channel.set_tag(channel_tag);
            channel.set_audio_type(audio_type);
            channel.set_volume(volume);

            Some(channel)
        } else {
            None
        }
    }

    /// Play specfied sound file.
    pub fn play_sound(&mut self, sound: &str, pos: Vec3<f32>) -> Option<usize> {
        if self.audio_device.is_some() {
            let calc_pos = ((pos - self.listener_pos) * FALLOFF).into_array();

            let sound = self.sound_cache.load_sound(sound);

            let left_ear = self.listener_ear_left.into_array();
            let right_ear = self.listener_ear_right.into_array();

            if let Some(channel) = self.get_channel(AudioType::Sfx, None) {
                channel.set_emitter_position(calc_pos);
                channel.set_left_ear_position(left_ear);
                channel.set_right_ear_position(right_ear);
                channel.play(sound);

                return Some(channel.get_id());
            }
        }

        None
    }

    pub fn play_music(&mut self, sound: &str, channel_tag: Option<ChannelTag>) -> Option<usize> {
        if self.audio_device.is_some() {
            if let Some(channel) = self.get_channel(AudioType::Music, channel_tag) {
                let file = assets::load_file(&sound, &["ogg"]).expect("Failed to load sound");
                let sound = Decoder::new(file).expect("Failed to decode sound");

                channel.set_emitter_position([0.0; 3]);
                channel.play(sound);

                return Some(channel.get_id());
            }
        }

        None
    }

    pub fn set_listener_pos(&mut self, pos: &Vec3<f32>, ori: &Vec3<f32>) {
        self.listener_pos = pos.clone();
        self.listener_ori = ori.normalized();

        let up = Vec3::new(0.0, 0.0, 1.0);

        let pos_left = up.cross(self.listener_ori.clone()).normalized();
        let pos_right = self.listener_ori.cross(up.clone()).normalized();

        self.listener_ear_left = pos_left;
        self.listener_ear_right = pos_right;

        for channel in self.channels.iter_mut() {
            if !channel.is_done() && channel.get_audio_type() == AudioType::Sfx {
                // TODO: Update this to correctly determine the updated relative position of
                // the SFX emitter when the player (listener) moves
                // channel.set_emitter_position(
                //     ((channel.pos - self.listener_pos) * FALLOFF).into_array(),
                // );
                channel.set_left_ear_position(pos_left.into_array());
                channel.set_right_ear_position(pos_right.into_array());
            }
        }
    }

    pub fn play_title_music(&mut self) -> Option<usize> {
        if self.music_enabled() {
            self.play_music(
                "voxygen.audio.soundtrack.veloren_title_tune",
                Some(ChannelTag::TitleMusic),
            )
        } else {
            None
        }
    }

    pub fn stop_title_music(&mut self) {
        let index = self.channels.iter().position(|c| {
            !c.is_done() && c.get_tag().is_some() && c.get_tag().unwrap() == ChannelTag::TitleMusic
        });

        if let Some(index) = index {
            self.channels[index].stop(Fader::fade_out(1.5, self.music_volume));
        }
    }

    pub fn stop_channel(&mut self, channel_id: usize, fader: Fader) {
        let index = self.channels.iter().position(|c| c.get_id() == channel_id);

        if let Some(index) = index {
            self.channels[index].stop(fader);
        }
    }

    pub fn get_sfx_volume(&self) -> f32 { self.sfx_volume }

    pub fn get_music_volume(&self) -> f32 { self.music_volume }

    pub fn sfx_enabled(&self) -> bool { self.sfx_volume > 0.0 }

    pub fn music_enabled(&self) -> bool { self.music_volume > 0.0 }

    pub fn set_sfx_volume(&mut self, sfx_volume: f32) {
        self.sfx_volume = sfx_volume;

        for channel in self.channels.iter_mut() {
            if channel.get_audio_type() == AudioType::Sfx {
                channel.set_volume(sfx_volume);
            }
        }
    }

    pub fn set_music_volume(&mut self, music_volume: f32) {
        self.music_volume = music_volume;

        for channel in self.channels.iter_mut() {
            if channel.get_audio_type() == AudioType::Music {
                if music_volume > 0.0 {
                    channel.set_volume(music_volume);
                } else {
                    channel.stop(Fader::fade_out(0.0, 0.0));
                }
            }
        }
    }

    // TODO: figure out how badly this will break things when it is called
    pub fn set_device(&mut self, name: String) {
        self.device = name.clone();
        self.audio_device = get_device_raw(&name);
    }
}

/// Returns the default audio device.
/// Does not return rodio Device struct in case our audio backend changes.
pub fn get_default_device() -> String {
    rodio::default_output_device()
        .expect("No audio output devices detected.")
        .name()
        .expect("Unable to get device name")
}

/// Returns a vec of the audio devices available.
/// Does not return rodio Device struct in case our audio backend changes.
pub fn list_devices() -> Vec<String> {
    list_devices_raw()
        .iter()
        .map(|x| x.name().expect("Unable to get device name"))
        .collect()
}

/// Returns vec of devices
fn list_devices_raw() -> Vec<Device> {
    rodio::output_devices()
        .expect("Unable to get output devices")
        .collect()
}

fn get_device_raw(device: &str) -> Option<Device> {
    rodio::output_devices()
        .expect("Unable to get output devices")
        .find(|d| d.name().expect("Unable to get device name") == device)
}
