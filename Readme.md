# Feig Notification Parser

Parses following data in following priority order from raw bytes:

1. Data (Tags)
1. Keepalive
1. Generic

## Todo

- [x] multiple tags in 1 message
- [x] keepalives
- [x] generics (fallback)
- [x] _understand_ (e.g. temperature alarm) keepalive codes
  - [ ] how to test with hw?
- [ ] handle feig date format (similar to franke date)
- [ ] remove additional Raw field when ready
  - [ ] remove also raw keepalive_flags when ready
- [x] add serde serialize to types
