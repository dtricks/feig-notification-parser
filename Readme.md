# Feig Notification Parser

Parses following data in following order from raw bytes:

1. Data (Tags)
1. Keepalive
1. Generic

## Todo

- [x] multiple tags in 1 message
- [x] keepalives
- [x] generics
- [ ] _understand_ (e.g. temperature alarm) keepalive codes
  - [ ] how to test?
- [ ] handle feig date format (similar to franke date)
