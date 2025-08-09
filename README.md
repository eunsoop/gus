# Git User Switcher (GUS)
Git User Switcher (GUS) is a simple command-line tool that allows you to switch between different Git user configurations easily. It is particularly useful for developers who work on multiple projects with different Git identities.

## RoadMap
- [x] Add support for switching between user profile
- [x] Add support for switching back to global profile
- [ ] Add support for creating new user profiles
- [ ] Sync profiles & ssh key between multiple devices 

## How does it work?
GUS changes .git/config file in the current directory to the specified user configuration. It can also switch back to the global configuration. <br/>
GUS stores user configurations in a toml file at `~/.gus/config`.

