cargo = find_program('cargo')

cargo_options = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', meson.current_build_dir() ]

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

workspace_members = [
  'lockinspiel-gtk'
]

foreach member: workspace_members
  package_build_dir = meson.current_build_dir()
  cargo_env.set('PACKAGE_BUILD_ROOT', package_build_dir / member)
  subdir(member)
endforeach
