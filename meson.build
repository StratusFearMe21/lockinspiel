project(
  'lockinspiel',
  'rust',
  version: '0.1.0',
  meson_version: '>= 0.59',
  license: 'GPL-3.0-or-later',
)

version = meson.project_version()

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')

meson.add_dist_script(
  'build-aux/dist-vendor.sh',
  meson.project_build_root() / 'meson-dist' / meson.project_name() + '-' + version,
  meson.project_source_root()
)

if get_option('debug')
  # Setup pre-commit hook for ensuring coding style is always consistent
  message('Setting up git pre-commit hook..')
  run_command('cp', '-f', meson.current_source_dir() / 'hooks/pre-commit.hook', meson.current_source_dir() / '.git/hooks/pre-commit', check: false)
endif


subdir('crates')
