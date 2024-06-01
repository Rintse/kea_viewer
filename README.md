# kea leases viewer

A simple HTTP server that serves up HTML pages containing the current leases of
the `kea` DHCP server.

![example page](./example.png)

## Configuration

The following environment variables can be defined to overwrite default
behaviour:
```bash
BIND_ADDR="0.0.0.0:80" # The socket address that the HTTP server binds to
LEASES_DB="/var/db/kea/dhcp4.leases" # The dhcp4.leases file location
# If LEASES_DB is a directory, all files in that directory will be used
RUST_LOG=debug # Additionally, use env_logger variables to configure logging
```
