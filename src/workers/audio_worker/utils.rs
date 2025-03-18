use libpulse_binding::context::{Context, State};
use libpulse_binding::error::PAErr;
use libpulse_binding::mainloop::standard::Mainloop;
use libpulse_binding::volume::{ChannelVolumes, Volume};
use std::sync::{Arc, Mutex};

// code is from https://github.com/de-vri-es/volume-ctl

/*
BSD 2-Clause License

Copyright (c) 2024, Maarten de Vries <maarten@de-vri.es>

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

pub fn get_output_volumes(
    main_loop: &mut Mainloop,
    context: &Context,
) -> Result<ChannelVolumes, PAErr> {
    run(main_loop, move |output| {
        context
            .introspect()
            .get_sink_info_by_name("@DEFAULT_SINK@", move |info| match info {
                libpulse_binding::callbacks::ListResult::Item(x) => {
                    *output.lock().unwrap() = Some(Ok(x.volume));
                }
                libpulse_binding::callbacks::ListResult::End => {}
                libpulse_binding::callbacks::ListResult::Error => {
                    *output.lock().unwrap() = Some(Err(()));
                }
            });
    })?
    .map_err(|()| context.errno())
}

pub fn connect(main_loop: &mut Mainloop) -> Result<Context, ()> {
    let mut context = libpulse_binding::context::Context::new(main_loop, "volume-control")
        .ok_or_else(|| eprintln!("Failed initialize PulseAudio context."))?;
    log::debug!("Protocol version: {}", context.get_protocol_version());
    log::debug!("Context state: {:?}", context.get_state());

    context
        .connect(None, libpulse_binding::context::FlagSet::NOFLAGS, None)
        .map_err(|e| eprintln!("Failed to connect to PulseAudio server: {e}"))?;
    log::debug!("Context state: {:?}", context.get_state());

    run_until(main_loop, |_main_loop| {
        let state = context.get_state();
        log::debug!("Context state: {:?}", state);
        match state {
            State::Ready => true,
            State::Failed => true,
            State::Unconnected => true,
            State::Terminated => true,
            State::Connecting => false,
            State::Authorizing => false,
            State::SettingName => false,
        }
    })
    .map_err(|e| log::error!("Error in PulseAudio main loop: {e}"))?;

    let state = context.get_state();
    match state {
        State::Ready => (),
        State::Failed => {
            log::error!(
                "Failed to connect to PulseAudio server: {}",
                context.errno()
            );
            return Err(());
        }
        State::Unconnected
        | State::Terminated
        | State::Connecting
        | State::Authorizing
        | State::SettingName => {
            log::error!("PulseAudio context in unexpected state: {state:?}");
            log::error!("Last error: {}", context.errno());
            return Err(());
        }
    }
    Ok(context)
}

pub fn run_until<F>(main_loop: &mut Mainloop, condition: F) -> Result<Option<i32>, PAErr>
where
    F: Fn(&mut Mainloop) -> bool,
{
    use libpulse_binding::mainloop::standard::IterateResult;
    loop {
        match main_loop.iterate(true) {
            IterateResult::Err(e) => {
                return Err(e);
            }
            IterateResult::Quit(code) => {
                return Ok(Some(code.0));
            }
            IterateResult::Success(_iterations) => (),
        }
        if condition(main_loop) {
            return Ok(None);
        };
    }
}

pub fn run<F, T>(main_loop: &mut Mainloop, operation: F) -> Result<T, PAErr>
where
    F: FnOnce(Arc<Mutex<Option<T>>>),
{
    use libpulse_binding::mainloop::standard::IterateResult;
    let output = Arc::new(Mutex::new(None));
    operation(output.clone());

    loop {
        if let Some(value) = output.lock().unwrap().take() {
            return Ok(value);
        }
        match main_loop.iterate(true) {
            IterateResult::Err(e) => {
                return Err(e);
            }
            IterateResult::Quit(code) => {
                std::process::exit(code.0);
            }
            IterateResult::Success(_iterations) => (),
        }
    }
}

pub fn volume_to_percentage(volume: Volume) -> f64 {
    let range = Volume::NORMAL.0 as f64 - Volume::MUTED.0 as f64;
    (volume.0 as f64 - Volume::MUTED.0 as f64) * 100.0 / range
}

pub fn set_output_volumes(
    main_loop: &mut Mainloop,
    context: &Context,
    volumes: &ChannelVolumes,
) -> Result<(), PAErr> {
    run(main_loop, move |output| {
        context.introspect().set_sink_volume_by_name(
            "@DEFAULT_SINK@",
            volumes,
            Some(Box::new(move |success| {
                if success {
                    *output.lock().unwrap() = Some(Ok(()));
                } else {
                    *output.lock().unwrap() = Some(Err(()));
                }
            })),
        );
    })?
    .map_err(|()| context.errno())
}

pub fn percentage_to_volume(factor: f64) -> Volume {
    let range = Volume::NORMAL.0 as f64 - Volume::MUTED.0 as f64;
    Volume((Volume::MUTED.0 as f64 + factor * range / 100.0) as u32)
}

pub fn map_volumes<F: FnMut(f64) -> f64>(volumes: &mut ChannelVolumes, mut action: F) {
    for volume in volumes.get_mut() {
        let factor = volume_to_percentage(*volume);
        let adjusted = action(factor).clamp(0.0, 125.0);
        *volume = percentage_to_volume(adjusted);
    }
}
