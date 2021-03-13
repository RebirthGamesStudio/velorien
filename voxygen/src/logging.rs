use std::fs;

use crate::settings::Settings;

use termcolor::{ColorChoice, StandardStream};
use tracing::{debug, error, info, trace};
use tracing_subscriber::{filter::LevelFilter, prelude::*, registry, EnvFilter};

const RUST_LOG_ENV: &str = "RUST_LOG";

/// Initialise tracing and logging for the settings.
///
/// This function will attempt to set up both a file and a terminal logger,
/// falling back to just a terminal logger if the file is unable to be created.
///
/// The logging level is by default set to `INFO`, to change this for any
/// particular crate or module you must use the `RUST_LOG` environment
/// variable.
///
/// For example to set this crate's debug level to `TRACE` you would need the
/// following in your environment.
/// `RUST_LOG="veloren_voxygen=trace"`
///
/// more complex tracing can be done by concatenating with a `,` as seperator:
///  - warn for `prometheus_hyper`, `dot_vox`, `gfx_device_gl::factory,
///    `gfx_device_gl::shade` trace for `veloren_voxygen`, info for everything
///    else
/// `RUST_LOG="prometheus_hyper=warn,dot_vox::parser=warn,gfx_device_gl::
/// factory=warn,gfx_device_gl::shade=warn,veloren_voxygen=trace,info"`
///
/// By default a few directives are set to `warn` by default, until explicitly
/// overwritten! e.g. `RUST_LOG="gfx_device_gl=debug"`
pub fn init(settings: &Settings) -> Vec<impl Drop> {
    // To hold the guards that we create, they will cause the logs to be
    // flushed when they're dropped.
    let mut _guards = vec![];

    // We will do lower logging than the default (INFO) by INCLUSION. This
    // means that if you need lower level logging for a specific module, then
    // put it in the environment in the correct format i.e. DEBUG logging for
    // this crate would be veloren_voxygen=debug.
    let base_exceptions = |env: EnvFilter| {
        env.add_directive("dot_vox::parser=warn".parse().unwrap())
            .add_directive("gfx_device_gl=warn".parse().unwrap())
            .add_directive("prometheus_hyper=warn".parse().unwrap())
            .add_directive("mio::sys::windows=info".parse().unwrap())
            .add_directive("h2=info".parse().unwrap())
            .add_directive("tokio_util=info".parse().unwrap())
            .add_directive("rustls=info".parse().unwrap())
            .add_directive("veloren_network_protocol=info".parse().unwrap())
            .add_directive(
                "veloren_server::persistence::character=info"
                    .parse()
                    .unwrap(),
            )
            .add_directive(LevelFilter::INFO.into())
    };

    #[cfg(not(feature = "tracy"))]
    let filter = match std::env::var_os(RUST_LOG_ENV).map(|s| s.into_string()) {
        Some(Ok(env)) => {
            let mut filter = base_exceptions(EnvFilter::new(""));
            for s in env.split(',').into_iter() {
                match s.parse() {
                    Ok(d) => filter = filter.add_directive(d),
                    Err(err) => println!("WARN ignoring log directive: `{}`: {}", s, err),
                };
            }
            filter
        },
        _ => base_exceptions(EnvFilter::from_env(RUST_LOG_ENV)),
    };

    #[cfg(feature = "tracy")]
    let filter = base_exceptions(EnvFilter::new("")).add_directive(LevelFilter::TRACE.into());

    // Create the terminal writer layer.
    let (non_blocking, _stdio_guard) =
        tracing_appender::non_blocking(StandardStream::stdout(ColorChoice::Auto));
    _guards.push(_stdio_guard);

    // Try to create the log file's parent folders.
    let log_folders_created = fs::create_dir_all(&settings.log.logs_path);
    const LOG_FILENAME: &str = "voxygen.log";

    match log_folders_created {
        // If the parent folders were created then attach both a terminal and a
        // file writer to the registry and init it.
        Ok(_) => {
            let file_appender =
                tracing_appender::rolling::daily(&settings.log.logs_path, LOG_FILENAME);
            let (non_blocking_file, _file_guard) = tracing_appender::non_blocking(file_appender);
            _guards.push(_file_guard);
            #[cfg(not(feature = "tracy"))]
            registry()
                .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
                .with(tracing_subscriber::fmt::layer().with_writer(non_blocking_file))
                .with(filter)
                .init();
            #[cfg(feature = "tracy")]
            registry()
                // NOTE: collecting stacks has a significant overhead (x6 overhead of
                // starting/stopping a span through the layer interface)
                .with(tracing_tracy::TracyLayer::new().with_stackdepth(0))
                .with(filter)
                .init();
            let logdir = &settings.log.logs_path;
            info!(?logdir, "Setup terminal and file logging.");
        },
        // Otherwise just add a terminal writer and init it.
        Err(e) => {
            error!(
                ?e,
                "Failed to create log file!. Falling back to terminal logging only.",
            );
            #[cfg(not(feature = "tracy"))]
            registry()
                .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
                .with(filter);
            #[cfg(feature = "tracy")]
            registry()
                .with(tracing_tracy::TracyLayer::new().with_stackdepth(0))
                .with(filter)
                .init();
            info!("Setup terminal logging.");
        },
    };
    debug!("Tracing is successfully set to DEBUG or TRACE");
    trace!("Tracing is successfully set to TRACE");

    // Return the guards
    _guards
}
