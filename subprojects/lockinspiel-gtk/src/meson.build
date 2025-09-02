global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('PKGDATADIR', pkgdatadir)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('VERSION', version + version_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR', localedir)
configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)

cargo_options = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', meson.project_build_root() / 'src' ]

if get_option('debug')
  rust_target = 'debug'
  message('Building in debug mode')
else
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
endif

cargo_options += [ '--', '--cfg', 'meson' ]

cargo_env = environment()

cargo_env.set('CARGO_HOME', meson.project_build_root() / 'cargo-home')
cargo_env.set('MESON_BUILD_ROOT', meson.project_build_root())

rustup = find_program('rustup', version : '>=1.18.3', required : false)

if rustup.found()
  rustup_home = run_command(rustup, 'show', 'home', capture : true, check : false)
  if rustup_home.returncode() == 0
    cargo_env.set('RUSTUP_HOME', rustup_home.stdout().strip())
  endif
endif

custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  depends: resources,
  env: cargo_env,
  command: [
    cargo, 'rustc',
    cargo_options,
    '&&',
    'cp', meson.project_build_root() / 'src' / rust_target / meson.project_name(), '@OUTPUT@',
  ]
)
