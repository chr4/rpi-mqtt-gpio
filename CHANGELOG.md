# Changelog

## v0.1.3

- Display version when running without arguments.
- Automatically resend current status upon reconnect.


## v0.1.2

- Add support for `availability_topic` to broadcast availability status via mqtt last will.


## v0.1.1

- Use `mqtt_state_low` and `mqtt_state_high` instead of `mqtt_state_on` and `mqtt_state_off`, as this allows an intuitive and practical "inverse" config.
- Fix an issue with `mqtt_state_low` and `mqtt_state_high` configuration values being ignored.
- Add option `initial_state`, where the initial state of an gpio pin can be configured.


## v0.1.0

- Initial feature set
